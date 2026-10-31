// Custom Dictionary: the words the user added themselves.

import SwiftUI
import UniformTypeIdentifiers

/// Multi-lect custom word entry aggregated across lects by Hanji ("四南海大平安").
struct CustomWordRow: Identifiable, Equatable {
    let id: String
    var hanji: String
    var xiienRow: CustomDictionaryRow?
    var namxiianRow: CustomDictionaryRow?
    var hoiliugRow: CustomDictionaryRow?
    var taibuRow: CustomDictionaryRow?
    var ngiaupinRow: CustomDictionaryRow?
    var zhioonRow: CustomDictionaryRow?

    var xiienRoman: String { xiienRow?.roman ?? "" }
    var namxiianRoman: String { namxiianRow?.roman ?? "" }
    var hoiliugRoman: String { hoiliugRow?.roman ?? "" }
    var taibuRoman: String { taibuRow?.roman ?? "" }
    var ngiaupinRoman: String { ngiaupinRow?.roman ?? "" }
    var zhioonRoman: String { zhioonRow?.roman ?? "" }

    /// Hakfa fork: the readings the word has, in the project's order 四南海大平安, for the table's one
    /// readings cell (`tag` is the one-character tag, `name` the lect's name for the tooltip).
    var readings: [(tag: String, name: String, roman: String)] {
        [
            ("四", "四縣", xiienRoman),
            ("南", "南四縣", namxiianRoman),
            ("海", "海陸", hoiliugRoman),
            ("大", "大埔", taibuRoman),
            ("平", "饒平", ngiaupinRoman),
            ("安", "詔安", zhioonRoman),
        ].filter { !$0.2.isEmpty }
    }

    var allRows: [CustomDictionaryRow] {
        [xiienRow, namxiianRow, hoiliugRow, taibuRow, ngiaupinRow, zhioonRow].compactMap { $0 }
    }

    mutating func addRow(_ row: CustomDictionaryRow) {
        switch row.lect {
        case 1: xiienRow = row
        case 6: namxiianRow = row
        case 2: hoiliugRow = row
        case 3: taibuRow = row
        case 4: ngiaupinRow = row
        case 5: zhioonRow = row
        case 0:
            if xiienRow == nil { xiienRow = row }
            if namxiianRow == nil { namxiianRow = row }
            if hoiliugRow == nil { hoiliugRow = row }
            if taibuRow == nil { taibuRow = row }
            if ngiaupinRow == nil { ngiaupinRow = row }
            if zhioonRow == nil { zhioonRow = row }
        default:
            if xiienRow == nil { xiienRow = row }
        }
    }
}

/// Reads and writes the custom dictionary on behalf of the page, one page of
/// rows at a time (`UserDataPagedList`).
@MainActor
@Observable
final class CustomDictionaryPageModel {
    private(set) var list = UserDataPagedList<CustomDictionaryRow>()
    private(set) var activity: UserDataPageActivity = .idle
    /// Once changed, makes every load in flight stale at once
    /// (`UserDataPagedList.invalidate`), not when the settled filter's own
    /// load starts.
    var filter = "" {
        didSet {
            if oldValue != filter {
                list.invalidate()
            }
        }
    }

    var message: UserDataPageMessage?

    /// Aggregates flat rows into multi-lect word rows by Hanji.
    var wordRows: [CustomWordRow] {
        var map: [String: CustomWordRow] = [:]
        var order: [String] = []

        for row in list.rows {
            let key = row.hanji.isEmpty ? row.id : row.hanji
            if var existing = map[key] {
                existing.addRow(row)
                map[key] = existing
            } else {
                var entry = CustomWordRow(id: key, hanji: row.hanji)
                entry.addRow(row)
                map[key] = entry
                order.append(key)
            }
        }
        return order.compactMap { map[$0] }
    }

    var selectedWordID: CustomWordRow.ID?

    func pageBackward() async {
        guard list.step(by: -1) else { return }
        await load()
    }

    func pageForward() async {
        guard list.step(by: 1) else { return }
        await load()
    }

    private let client: any UserDataClient

    init(client: any UserDataClient) {
        self.client = client
    }

    /// Reloads the page on screen.
    func load() async {
        let load = list.beginLoad()
        let (filter, offset) = (filter, load.offset)
        do {
            let listing = try await UserDataRequests.run(on: client) {
                try $0.list(filter: filter, limit: UserDataListMetrics.pageSize, offset: offset)
            }
            list.land(listing, from: load)
        } catch {
            guard list.isCurrent(load) else { return }
            message = .failure(.desktopCustomDictReadFailed, error)
        }
    }

    /// Back to page one, then load. What a filter change asks for.
    func loadFirstPage() async {
        list.rewind()
        await load()
    }

    var selectedRowID: CustomDictionaryRow.ID? {
        get { list.selectedID }
        set { list.selectedID = newValue }
    }

    func save(_ row: CustomDictionaryRow) async {
        await perform(.desktopProgressWorking) { try await UserDataRequests.run(on: self.client) { try $0.save(row) } }
    }

    func delete(_ row: CustomDictionaryRow) async {
        let id = row.id
        await perform(.desktopProgressWorking) { try await UserDataRequests.run(on: self.client) { try $0.delete(id: id) } }
    }

    func deleteWord(_ word: CustomWordRow) async {
        let rows = word.allRows
        await perform(.desktopProgressWorking) {
            for row in rows {
                try await UserDataRequests.run(on: self.client) { try $0.delete(id: row.id) }
            }
        }
    }

    func saveWord(
        hanji: String,
        xiien: String,
        namxiian: String,
        hoiliug: String,
        taibu: String,
        ngiaupin: String,
        zhioon: String,
        original: CustomWordRow?
    ) async {
        let trimmedHanji = hanji.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedXiien = xiien.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedNamxiian = namxiian.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedHoiliug = hoiliug.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedTaibu = taibu.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedNgiaupin = ngiaupin.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedZhioon = zhioon.trimmingCharacters(in: .whitespacesAndNewlines)

        await perform(.desktopProgressWorking) {
            let entries: [(String, CustomDictionaryRow?, UInt32)] = [
                (trimmedXiien, original?.xiienRow, 1),
                (trimmedNamxiian, original?.namxiianRow, 6),
                (trimmedHoiliug, original?.hoiliugRow, 2),
                (trimmedTaibu, original?.taibuRow, 3),
                (trimmedNgiaupin, original?.ngiaupinRow, 4),
                (trimmedZhioon, original?.zhioonRow, 5),
            ]
            for (roman, oldRow, lect) in entries {
                if !roman.isEmpty {
                    let id = oldRow?.id ?? UUID().uuidString
                    let row = CustomDictionaryRow(id: id, roman: roman, hanji: trimmedHanji, lect: lect)
                    try await UserDataRequests.run(on: self.client) { try $0.save(row) }
                } else if let old = oldRow {
                    try await UserDataRequests.run(on: self.client) { try $0.delete(id: old.id) }
                }
            }
        }
    }

    /// Runs on the click, no confirmation (USER 2026-10-16).
    func deleteAll() async {
        await perform(.desktopProgressWorking) { try await UserDataRequests.run(on: self.client) { try $0.deleteAll() } }
    }

    func exportCSV(in window: NSWindow) async {
        guard beginWork(.desktopProgressWorking) else { return }
        defer { activity = .idle }
        do {
            let csv = try await UserDataRequests.run(on: client) { try $0.exportCSV() }
            _ = try await UserDataFilePanels.write(
                csv,
                suggestedName: UserDataFilePanels.exportFileName(
                    prefix: "taigi_custom_dictionary",
                    extension: "csv",
                ),
                contentTypes: [.commaSeparatedText],
                in: window,
            )
        } catch {
            message = .failure(.commonExportFailed, error)
        }
    }

    func importCSV(in window: NSWindow) async {
        guard beginWork(.desktopProgressWorking) else { return }
        defer { activity = .idle }
        guard let url = await UserDataFilePanels.chooseFileToOpen(
            contentTypes: [.commaSeparatedText, .plainText],
            in: window,
        ) else { return }
        await importCSV(at: url)
    }

    func importCSV(at url: URL) async {
        do {
            let result = try await UserDataRequests.run(on: client) { try $0.importCSV(at: url) }
            message = .imported(result.imported, skipped: result.skipped)
        } catch {
            message = .failure(.commonImportFailed, error)
        }
        await load()
    }

    func beginWork(_ label: StringKey) -> Bool {
        activity.begin(label)
    }

    private func perform(_ label: StringKey, _ body: () async throws -> Void) async {
        guard beginWork(label) else { return }
        defer { activity = .idle }
        do {
            try await body()
        } catch {
            message = .failure(.desktopCustomDictWriteFailed, error)
        }
        await load()
    }
}

struct CustomDictionaryPage: View {
    @Environment(DisplayLanguageStore.self) private var language

    @State private var model: CustomDictionaryPageModel
    @State private var editing: CustomWordRow?
    @AppStorage(SettingsStore.Keys.isCustomDictEnabled.name)
    private var isCustomDictEnabled = SettingsStore.Keys.isCustomDictEnabled.defaultValue

    @AppStorage(SettingsStore.Keys.lectEnabled(.xiien).name)
    private var isXiienEnabled = SettingsStore.Keys.lectEnabled(.xiien).defaultValue
    @AppStorage(SettingsStore.Keys.lectEnabled(.namxiian).name)
    private var isNamxiianEnabled = SettingsStore.Keys.lectEnabled(.namxiian).defaultValue
    @AppStorage(SettingsStore.Keys.lectEnabled(.hoiliug).name)
    private var isHoiliugEnabled = SettingsStore.Keys.lectEnabled(.hoiliug).defaultValue
    @AppStorage(SettingsStore.Keys.lectEnabled(.taibu).name)
    private var isTaibuEnabled = SettingsStore.Keys.lectEnabled(.taibu).defaultValue
    @AppStorage(SettingsStore.Keys.lectEnabled(.ngiaupin).name)
    private var isNgiaupinEnabled = SettingsStore.Keys.lectEnabled(.ngiaupin).defaultValue
    @AppStorage(SettingsStore.Keys.lectEnabled(.zhioon).name)
    private var isZhioonEnabled = SettingsStore.Keys.lectEnabled(.zhioon).defaultValue

    init(client: any UserDataClient) {
        _model = State(initialValue: CustomDictionaryPageModel(client: client))
    }

    var body: some View {
        Form {
            Section {
                Toggle(language.string(.dictionaryCustomDictEnabled), isOn: $isCustomDictEnabled)
            }

            Section {
                UserDataFilterField(text: $model.filter)
                entryTable
                entryTableControls
            } header: {
                HStack {
                    Text(language.string(.desktopEntriesSection))
                    Spacer()
                    Text(model.list.countLabel)
                        .foregroundStyle(.secondary)
                }
            }

            UserDataActionsSection(
                exportTitle: .dictionaryExportCSV,
                importTitle: .dictionaryImportCSV,
                deleteTitle: .dictionaryDeleteAll,
                onExport: { Task { await UserDataFilePanels.withSettingsWindow(model.exportCSV) } },
                onImport: { Task { await UserDataFilePanels.withSettingsWindow(model.importCSV) } },
                onDelete: { Task { await model.deleteAll() } },
            )
        }
        .formStyle(.grouped)
        .reloadWhenFilterSettles(model.filter) { await model.loadFirstPage() }
        .sheet(item: $editing) { word in
            CustomDictionaryEntrySheet(
                word: word,
                isXiienEnabled: isXiienEnabled,
                isNamxiianEnabled: isNamxiianEnabled,
                isHoiliugEnabled: isHoiliugEnabled,
                isTaibuEnabled: isTaibuEnabled,
                isNgiaupinEnabled: isNgiaupinEnabled,
                isZhioonEnabled: isZhioonEnabled
            ) { hanji, xiien, namxiian, hoiliug, taibu, ngiaupin, zhioon in
                Task {
                    await model.saveWord(
                        hanji: hanji,
                        xiien: xiien,
                        namxiian: namxiian,
                        hoiliug: hoiliug,
                        taibu: taibu,
                        ngiaupin: ngiaupin,
                        zhioon: zhioon,
                        original: word.id.isEmpty ? nil : word
                    )
                }
            }
        }
        .userDataPageChrome(activity: model.activity, message: $model.message)
    }

    /// Hakfa fork: every lect's reading in one cell — six per-lect columns do not fit the 545-point pane
    /// (the window cannot be widened), and a table cell is not edited in place anyway (a row opens the
    /// sheet). Two lines: `四 南 海` on the first, `大 平 安` on the second (a line with no reading is
    /// left out); a long reading shortens, and hover lists them by lect name.
    @ViewBuilder
    private func readingsCell(_ word: CustomWordRow) -> some View {
        let readings = word.readings
        if readings.isEmpty {
            Text("＋")
                .foregroundStyle(.tertiary)
        } else {
            let first = readings.filter { ["四", "南", "海"].contains($0.tag) }
            let second = readings.filter { ["大", "平", "安"].contains($0.tag) }
            VStack(alignment: .leading, spacing: 1) {
                if !first.isEmpty {
                    Text(first.map { "\($0.tag) \($0.roman)" }.joined(separator: "　"))
                }
                if !second.isEmpty {
                    Text(second.map { "\($0.tag) \($0.roman)" }.joined(separator: "　"))
                }
            }
            .foregroundStyle(.secondary)
            .lineLimit(1)
            .truncationMode(.tail)
            .padding(.vertical, 2)
            .help(readings.map { "\($0.name)　\($0.roman)" }.joined(separator: "\n"))
        }
    }

    private var entryTable: some View {
        Table(model.wordRows, selection: $model.selectedWordID) {
            TableColumn("諸夏字") { word in
                Text(word.hanji.isEmpty ? "—" : word.hanji)
                    .fontWeight(.medium)
                    .lineLimit(1)
            }
            .width(min: 56, ideal: 70)

            TableColumn("各腔讀音（四南海大平安）") { word in
                readingsCell(word)
            }
        }
        .tableStyle(.inset)
        .alternatingRowBackgrounds(.disabled)
        .frame(height: UserDataListMetrics.pagedTableHeight)
        .overlay {
            if model.wordRows.isEmpty {
                emptyState
            }
        }
        .contextMenu(forSelectionType: CustomWordRow.ID.self) { ids in
            if let word = word(for: ids.first) {
                Button(language.string(.dictionaryEditEntry)) { editing = word }
                Button(language.string(.commonDelete), role: .destructive) {
                    Task { await model.deleteWord(word) }
                }
            }
        } primaryAction: { ids in
            editing = word(for: ids.first)
        }
    }

    @ViewBuilder
    private var emptyState: some View {
        if model.filter.isEmpty {
            UserDataListEmptySymbol(
                symbolName: UserDataListMetrics.emptyStateSymbolName,
                accessibilityLabelKey: .dictionaryCustomDictEmpty,
            )
        } else {
            Text(language.string(.dictionaryNoResults))
                .foregroundStyle(.secondary)
        }
    }

    private var entryTableControls: some View {
        UserDataListControls(
            add: (.dictionaryAddEntry, { editing = CustomWordRow(id: "", hanji: "") }),
            isRemoveEnabled: selectedWord != nil,
            onRemove: {
                guard let word = selectedWord else { return }
                Task { await model.deleteWord(word) }
            },
        ) {
            UserDataListPager(
                page: model.list.page,
                pageCount: model.list.pageCount,
                onBackward: { Task { await model.pageBackward() } },
                onForward: { Task { await model.pageForward() } },
            )
        }
    }

    private var selectedWord: CustomWordRow? {
        word(for: model.selectedWordID)
    }

    private func word(for id: CustomWordRow.ID?) -> CustomWordRow? {
        model.wordRows.first { $0.id == id }
    }
}

/// Add or edit one custom dictionary entry with multi-lect columns.
struct CustomDictionaryEntrySheet: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(DisplayLanguageStore.self) private var language

    @State private var hanji: String
    @State private var xiien: String
    @State private var namxiian: String
    @State private var hoiliug: String
    @State private var taibu: String
    @State private var ngiaupin: String
    @State private var zhioon: String

    private let original: CustomWordRow
    private let isXiienEnabled: Bool
    private let isNamxiianEnabled: Bool
    private let isHoiliugEnabled: Bool
    private let isTaibuEnabled: Bool
    private let isNgiaupinEnabled: Bool
    private let isZhioonEnabled: Bool
    private let onSave: (String, String, String, String, String, String, String) -> Void

    init(
        word: CustomWordRow,
        isXiienEnabled: Bool,
        isNamxiianEnabled: Bool,
        isHoiliugEnabled: Bool,
        isTaibuEnabled: Bool,
        isNgiaupinEnabled: Bool,
        isZhioonEnabled: Bool,
        onSave: @escaping (String, String, String, String, String, String, String) -> Void
    ) {
        original = word
        self.isXiienEnabled = isXiienEnabled
        self.isNamxiianEnabled = isNamxiianEnabled
        self.isHoiliugEnabled = isHoiliugEnabled
        self.isTaibuEnabled = isTaibuEnabled
        self.isNgiaupinEnabled = isNgiaupinEnabled
        self.isZhioonEnabled = isZhioonEnabled
        self.onSave = onSave
        _hanji = State(initialValue: word.hanji)
        _xiien = State(initialValue: word.xiienRoman)
        _namxiian = State(initialValue: word.namxiianRoman)
        _hoiliug = State(initialValue: word.hoiliugRoman)
        _taibu = State(initialValue: word.taibuRoman)
        _ngiaupin = State(initialValue: word.ngiaupinRoman)
        _zhioon = State(initialValue: word.zhioonRoman)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text(original.id.isEmpty ? language.string(.dictionaryAddEntry) : language.string(.dictionaryEditEntry))
                .font(.headline)
            Form {
                TextField("諸夏字", text: $hanji)
                if isXiienEnabled {
                    TextField("打四縣", text: $xiien)
                }
                if isNamxiianEnabled {
                    TextField("打南四縣", text: $namxiian)
                }
                if isHoiliugEnabled {
                    TextField("打海陸", text: $hoiliug)
                }
                if isTaibuEnabled {
                    TextField("打大埔", text: $taibu)
                }
                if isNgiaupinEnabled {
                    TextField("打饒平", text: $ngiaupin)
                }
                if isZhioonEnabled {
                    TextField("打詔安", text: $zhioon)
                }
            }
            .formStyle(.grouped)
            .frame(minHeight: 120, maxHeight: 360)

            HStack {
                Button("各腔自動補音") {
                    autoFillReadings()
                }
                .disabled(hanji.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                Spacer()
                Button(language.string(.commonCancel), role: .cancel) { dismiss() }
                Button(language.string(.commonSave)) {
                    onSave(hanji, xiien, namxiian, hoiliug, taibu, ngiaupin, zhioon)
                    dismiss()
                }
                .keyboardShortcut(.defaultAction)
                .disabled(!hasAtLeastOneReading)
            }
        }
        .padding(20)
        .frame(width: 420)
    }

    private var hasAtLeastOneReading: Bool {
        [xiien, namxiian, hoiliug, taibu, ngiaupin, zhioon].contains {
            !$0.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
        }
    }

    private func autoFillReadings() {
        let text = hanji.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return }

        if isXiienEnabled && xiien.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            if let reading = lookupUniqueReading(hanji: text, lect: 1) {
                xiien = reading
            }
        }
        if isNamxiianEnabled && namxiian.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            if let reading = lookupUniqueReading(hanji: text, lect: 6) {
                namxiian = reading
            }
        }
        if isHoiliugEnabled && hoiliug.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            if let reading = lookupUniqueReading(hanji: text, lect: 2) {
                hoiliug = reading
            }
        }
        if isTaibuEnabled && taibu.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            if let reading = lookupUniqueReading(hanji: text, lect: 3) {
                taibu = reading
            }
        }
        if isNgiaupinEnabled && ngiaupin.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            if let reading = lookupUniqueReading(hanji: text, lect: 4) {
                ngiaupin = reading
            }
        }
        if isZhioonEnabled && zhioon.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            if let reading = lookupUniqueReading(hanji: text, lect: 5) {
                zhioon = reading
            }
        }
    }

    private func lookupUniqueReading(hanji: String, lect: UInt32) -> String? {
        var toggles = Taigi_Engine_DictionarySourceToggles()
        toggles.lect = lect
        toggles.kautian = true
        toggles.taigitv = true
        toggles.kungge = true

        var filterReq = Taigi_Engine_DictionaryFiltersRequest()
        filterReq.toggles = toggles

        guard case let .lexicon(filterResp)? = RustEngineBridge.roundtrip(
            payload: .lexicon(Taigi_Engine_LexiconRequest.with { $0.dictionaryFilters = filterReq }),
            op: "dictionaryFilters"
        ) else { return nil }
        guard case let .dictionaryFiltersResult(filters) = filterResp.result else { return nil }

        var req = Taigi_Engine_SearchByHanjiRequest()
        req.query = hanji
        req.limit = 20
        req.enabledSourcesBitmask = filters.dictionaryFilterBitmask

        guard case let .lexicon(resp)? = RustEngineBridge.roundtrip(
            payload: .lexicon(Taigi_Engine_LexiconRequest.with { $0.searchByHanji = req }),
            op: "lookupUniqueReading"
        ) else { return nil }
        guard case let .searchByHanjiResult(result) = resp.result else { return nil }
        let exactMatches = result.rows.filter { $0.hanji == hanji }
        let distinctReadings = Set(exactMatches.map { $0.roman.trimmingCharacters(in: .whitespacesAndNewlines) }.filter { !$0.isEmpty })
        return distinctReadings.count == 1 ? distinctReadings.first : nil
    }
}
