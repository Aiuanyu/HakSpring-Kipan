// What the settings pages ask of the user's data, which the engine owns.

import Foundation

/// One page of a user-data list — custom words or learning records — and
/// the two counts the page shows.
struct UserDataListing<Row: Sendable>: Sendable {
    let rows: [Row]
    /// Every stored row.
    let total: Int
    /// The rows the filter matches, for paging.
    let matchingTotal: Int
    /// Where `rows` start: the requested offset, pulled back to the last
    /// page by the engine when the matches shrank under it.
    let offset: Int
}

struct CustomDictionaryImportResult: Equatable, Sendable {
    let imported: Int
    let skipped: Int
}

/// Why a user-data request did nothing. The description is the alert's
/// diagnostic line, English on purpose (`UserDataPageMessage.failure`).
enum UserDataClientError: Error, Equatable, CustomStringConvertible {
    /// The round-trip failed or the engine refused the request; the bridge
    /// logged why.
    case engineUnavailable(op: String)
    /// Something the user can be told — a full dictionary, an unusable file —
    /// in the engine's own words (`custom dictionary is full (max 30000
    /// entries)`).
    case refused(detail: String)
    /// A reset that emptied some stores and not others, one line per store
    /// that failed (`user_frequency: <error>`).
    case notEmptied([String])

    var description: String {
        switch self {
        case let .engineUnavailable(op): "the engine did not answer \(op)"
        case let .refused(detail): detail
        case let .notEmptied(failures): failures.joined(separator: "\n")
        }
    }
}

/// The user-data requests the settings pages make. Synchronous: every call
/// is an engine round-trip that may touch SQLite, so callers run it off the
/// main actor. A protocol so the page's paging and work-slot logic can be
/// driven from tests without opening the process-wide user data — the test
/// process shares one engine, and an open there would reach every later
/// fetch in the run.
protocol UserDataClient: Sendable {
    func list(filter: String, limit: Int, offset: Int) throws -> UserDataListing<CustomDictionaryRow>
    func save(_ row: CustomDictionaryRow) throws
    func delete(id: String) throws
    /// Empties the custom dictionary.
    func deleteAll() throws
    func exportCSV() throws -> Data
    func importCSV(at url: URL) throws -> CustomDictionaryImportResult
    /// Empties what the input method learned — counts, bigrams, learned
    /// phrases — and leaves the custom dictionary alone.
    func clearLearningRecords() throws

    /// One page of one learning store, in `order`. The rows are the
    /// engine's own records, handed back whole by the two writes below: the
    /// engine applies a write only while the row still holds what was listed.
    func listLearningRecords(
        kind: Taigi_Engine_LearningRecordKind,
        order: Taigi_Engine_LearningRecordOrder,
        filter: String,
        limit: Int,
        offset: Int,
    ) throws -> UserDataListing<Taigi_Engine_LearningRecord>
    /// False when the row is gone — deleted, evicted, or its id taken by
    /// another word since it was listed.
    func setLearningRecordCount(_ record: Taigi_Engine_LearningRecord, count: Int) throws -> Bool
    /// False when the row is gone, as for `setLearningRecordCount`.
    func deleteLearningRecord(_ record: Taigi_Engine_LearningRecord) throws -> Bool
    /// Adds a row's word to the custom dictionary — unless the word is stored
    /// there already — then forgets a learned phrase (a frequency row stays).
    /// A refusal (a full dictionary,
    /// a reading that cannot be searched) keeps the row and is thrown as
    /// `UserDataClientError.refused`.
    func addLearningRecordToCustomDictionary(_ record: Taigi_Engine_LearningRecord) throws
    /// Hakfa fork: 各腔互學 — what each lect learned is offered to the others where the target
    /// lect's dictionary reads the same Hanji one way. `dryRun` only counts.
    func syncLects(dryRun: Bool) throws -> Taigi_Engine_LectsSynced
}

extension UserDataClient {
    /// Hakfa fork (D5): this client acting on the learning stores of `lect` (a lect code). A client
    /// with no lects (a test double) is its own answer.
    func scoped(to _: UInt32) -> any UserDataClient { self }

    /// How much each lect has learned; a client that cannot say reports nothing.
    func countLearning() throws -> [Taigi_Engine_LectLearningCount] { [] }

    /// Doubles that do not exercise it need not state it.
    func syncLects(dryRun _: Bool) throws -> Taigi_Engine_LectsSynced {
        throw UserDataClientError.engineUnavailable(op: "syncLects")
    }
}

/// The shipped client: the engine's user-data ops.
struct EngineUserDataClient: UserDataClient {
    /// Hakfa fork (D5): the lect whose learning stores the learning-record calls act on
    /// (0 = 四縣, the original files).
    var lect: UInt32 = 0

    func scoped(to lect: UInt32) -> any UserDataClient {
        EngineUserDataClient(lect: lect)
    }

    func countLearning() throws -> [Taigi_Engine_LectLearningCount] {
        guard let counts = RustEngineBridge.countLearning() else {
            throw UserDataClientError.engineUnavailable(op: "countLearning")
        }
        return counts.lects
    }

    /// The largest file an import reads. CROSS-PLATFORM INVARIANT — the
    /// engine refuses the same size (`engine/userdata/src/csv.rs`); checked
    /// here too so a huge file is refused before it is read into memory.
    static let maxImportFileBytes = 5 * 1024 * 1024

    func list(filter: String, limit: Int, offset: Int) throws -> UserDataListing<CustomDictionaryRow> {
        guard let page = RustEngineBridge.customDictionaryList(
            filter: filter,
            limit: limit,
            offset: offset,
        ) else { throw UserDataClientError.engineUnavailable(op: "customDictionaryList") }
        return UserDataListing(
            rows: page.entries.map(CustomDictionaryRow.init),
            total: Int(page.total),
            matchingTotal: Int(page.matchingTotal),
            offset: Int(page.offset),
        )
    }

    func save(_ row: CustomDictionaryRow) throws {
        guard let saved = RustEngineBridge.customDictionarySave(
            id: row.id,
            roman: row.roman,
            hanji: row.hanji,
        ) else { throw UserDataClientError.engineUnavailable(op: "customDictionarySave") }
        guard saved.refusal == .none else { throw UserDataClientError.refused(detail: saved.detail) }
    }

    func delete(id: String) throws {
        guard RustEngineBridge.customDictionaryDelete(id: id) != nil else {
            throw UserDataClientError.engineUnavailable(op: "customDictionaryDelete")
        }
    }

    func deleteAll() throws {
        try reset { $0.customDictionary = true }
    }

    func exportCSV() throws -> Data {
        guard let csv = RustEngineBridge.customDictionaryExportCSV() else {
            throw UserDataClientError.engineUnavailable(op: "customDictionaryExportCSV")
        }
        return csv
    }

    func importCSV(at url: URL) throws -> CustomDictionaryImportResult {
        let size = try url.resourceValues(forKeys: [.fileSizeKey]).fileSize ?? 0
        guard size <= Self.maxImportFileBytes else {
            throw UserDataClientError.refused(detail: "file is larger than 5 MB")
        }
        guard let imported = try RustEngineBridge.customDictionaryImportCSV(Data(contentsOf: url)) else {
            throw UserDataClientError.engineUnavailable(op: "customDictionaryImportCSV")
        }
        guard imported.refusal == .none else { throw UserDataClientError.refused(detail: imported.detail) }
        return CustomDictionaryImportResult(
            imported: Int(imported.imported),
            skipped: Int(imported.skipped),
        )
    }

    func syncLects(dryRun: Bool) throws -> Taigi_Engine_LectsSynced {
        guard let synced = RustEngineBridge.syncLects(dryRun: dryRun) else {
            throw UserDataClientError.engineUnavailable(op: "syncLects")
        }
        return synced
    }

    func clearLearningRecords() throws {
        try reset {
            $0.frequency = true
            $0.association = true
            $0.learnedPhrases = true
            $0.lect = lect
        }
    }

    func listLearningRecords(
        kind: Taigi_Engine_LearningRecordKind,
        order: Taigi_Engine_LearningRecordOrder,
        filter: String,
        limit: Int,
        offset: Int,
    ) throws -> UserDataListing<Taigi_Engine_LearningRecord> {
        guard let page = RustEngineBridge.learningRecordsList(
            kind: kind,
            order: order,
            filter: filter,
            limit: limit,
            offset: offset,
            lect: lect,
        ) else { throw UserDataClientError.engineUnavailable(op: "learningRecordsList") }
        return UserDataListing(
            rows: page.records,
            total: Int(page.total),
            matchingTotal: Int(page.matchingTotal),
            offset: Int(page.offset),
        )
    }

    func setLearningRecordCount(_ record: Taigi_Engine_LearningRecord, count: Int) throws -> Bool {
        guard let saved = RustEngineBridge.learningRecordSetCount(record, count: count, lect: lect) else {
            throw UserDataClientError.engineUnavailable(op: "learningRecordSetCount")
        }
        return saved.hasRecord
    }

    func deleteLearningRecord(_ record: Taigi_Engine_LearningRecord) throws -> Bool {
        guard let removed = RustEngineBridge.learningRecordDelete(record, lect: lect) else {
            throw UserDataClientError.engineUnavailable(op: "learningRecordDelete")
        }
        return removed
    }

    func addLearningRecordToCustomDictionary(_ record: Taigi_Engine_LearningRecord) throws {
        guard let added = RustEngineBridge.learningRecordAddToCustomDictionary(record, lect: lect) else {
            throw UserDataClientError.engineUnavailable(op: "learningRecordAddToCustomDictionary")
        }
        guard added.refusal == .none else { throw UserDataClientError.refused(detail: added.detail) }
    }

    /// Empties the stores `select` names; every one is attempted, and the
    /// ones that could not be emptied are reported together.
    private func reset(_ select: (inout Taigi_Engine_ResetUserData) -> Void) throws {
        var request = Taigi_Engine_ResetUserData()
        select(&request)
        guard let removed = RustEngineBridge.userDataReset(request) else {
            throw UserDataClientError.engineUnavailable(op: "userDataReset")
        }
        guard removed.failures.isEmpty else { throw UserDataClientError.notEmptied(removed.failures) }
    }
}

extension CustomDictionaryRow {
    init(_ entry: Taigi_Engine_CustomDictionaryEntry) {
        self.init(id: entry.id, roman: entry.roman, hanji: entry.hanji)
    }
}

/// A learned row is addressed by the store's row id on the page that listed
/// it — a table selection, never the word's identity (`(text, tl)`, which the
/// engine checks on every write).
extension Taigi_Engine_LearningRecord: Identifiable {}
