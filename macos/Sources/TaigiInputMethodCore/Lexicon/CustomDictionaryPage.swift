// Custom Dictionary: the words the user added themselves.

import SwiftUI
import UniformTypeIdentifiers

/// Multi-lect custom word entry aggregated across lects by Hanji.
struct CustomWordRow: Identifiable, Equatable {
    let id: String
    var hanji: String
    var xiienRow: CustomDictionaryRow?
    var namxiianRow: CustomDictionaryRow?
    var hoiliugRow: CustomDictionaryRow?

    var xiienRoman: String { xiienRow?.roman ?? "" }
    var namxiianRoman: String { namxiianRow?.roman ?? "" }
    var hoiliugRoman: String { hoiliugRow?.roman ?? "" }

    var allRows: [CustomDictionaryRow] {
        [xiienRow, namxiianRow, hoiliugRow].compactMap { $0 }
    }

    mutating func addRow(_ row: CustomDictionaryRow) {
        switch row.lect {
        case 1: xiienRow = row
        case 6: namxiianRow = row
        case 2: hoiliugRow = row
        case 0:
            if xiienRow == nil { xiienRow = row }
            if namxiianRow == nil { namxiianRow = row }
            if hoiliugRow == nil { hoiliugRow = row }
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
        original: CustomWordRow?
    ) async {
        let trimmedHanji = hanji.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedXiien = xiien.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedNamxiian = namxiian.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedHoiliug = hoiliug.trimmingCharacters(in: .whitespacesAndNewlines)

        await perform(.desktopProgressWorking) {
            // Xiien (lect 1)
            if !trimmedXiien.isEmpty {
                let id = original?.xiienRow?.id ?? UUID().uuidString
                let row = CustomDictionaryRow(id: id, roman: trimmedXiien, hanji: trimmedHanji, lect: 1)
                try await UserDataRequests.run(on: self.client) { try $0.save(row) }
            } else if let old = original?.xiienRow {
                try await UserDataRequests.run(on: self.client) { try $0.delete(id: old.id) }
            }

            // Namxiian (lect 6)
            if !trimmedNamxiian.isEmpty {
                let id = original?.namxiianRow?.id ?? UUID().uuidString
                let row = CustomDictionaryRow(id: id, roman: trimmedNamxiian, hanji: trimmedHanji, lect: 6)
                try await UserDataRequests.run(on: self.client) { try $0.save(row) }
            } else if let old = original?.namxiianRow {
                try await UserDataRequests.run(on: self.client) { try $0.delete(id: old.id) }
            }

            // Hoiliug (lect 2)
            if !trimmedHoiliug.isEmpty {
                let id = original?.hoiliugRow?.id ?? UUID().uuidString
                let row = CustomDictionaryRow(id: id, roman: trimmedHoiliug, hanji: trimmedHanji, lect: 2)
                try await UserDataRequests.run(on: self.client) { try $0.save(row) }
            } else if let old = original?.hoiliugRow {
                try await UserDataRequests.run(on: self.client) { try $0.delete(id: old.id) }
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
                isHoiliugEnabled: isHoiliugEnabled
            ) { hanji, xiien, namxiian, hoiliug in
                Task {
                    await model.saveWord(
                        hanji: hanji,
                        xiien: xiien,
                        namxiian: namxiian,
                        hoiliug: hoiliug,
                        original: word.id.isEmpty ? nil : word
                    )
                }
            }
        }
        .userDataPageChrome(activity: model.activity, message: $model.message)
    }

    private var entryTable: some View {
        Table(model.wordRows, selection: $model.selectedWordID) {
            TableColumn("諸夏字") { word in
                Text(word.hanji.isEmpty ? "—" : word.hanji)
                    .fontWeight(.medium)
            }
            .width(min: 80, ideal: 100)

            TableColumn("打四縣") { word in
                if word.xiienRoman.isEmpty {
                    Text("＋")
                        .foregroundStyle(.tertiary)
                } else {
                    Text(word.xiienRoman)
                        .foregroundStyle(.secondary)
                }
            }

            TableColumn("打南四縣") { word in
                if word.namxiianRoman.isEmpty {
                    Text("＋")
                        .foregroundStyle(.tertiary)
                } else {
                    Text(word.namxiianRoman)
                        .foregroundStyle(.secondary)
                }
            }

            TableColumn("打海陸") { word in
                if word.hoiliugRoman.isEmpty {
                    Text("＋")
                        .foregroundStyle(.tertiary)
                } else {
                    Text(word.hoiliugRoman)
                        .foregroundStyle(.secondary)
                }
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

    private let original: CustomWordRow
    private let isXiienEnabled: Bool
    private let isNamxiianEnabled: Bool
    private let isHoiliugEnabled: Bool
    private let onSave: (String, String, String, String) -> Void

    init(
        word: CustomWordRow,
        isXiienEnabled: Bool,
        isNamxiianEnabled: Bool,
        isHoiliugEnabled: Bool,
        onSave: @escaping (String, String, String, String) -> Void
    ) {
        original = word
        self.isXiienEnabled = isXiienEnabled
        self.isNamxiianEnabled = isNamxiianEnabled
        self.isHoiliugEnabled = isHoiliugEnabled
        self.onSave = onSave
        _hanji = State(initialValue: word.hanji)
        _xiien = State(initialValue: word.xiienRoman)
        _namxiian = State(initialValue: word.namxiianRoman)
        _hoiliug = State(initialValue: word.hoiliugRoman)
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
            }
            .formStyle(.grouped)

            HStack {
                Button("各腔自動補音") {
                    autoFillReadings()
                }
                .disabled(hanji.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)

                Spacer()
                Button(language.string(.commonCancel), role: .cancel) { dismiss() }
                Button(language.string(.commonSave)) {
                    onSave(hanji, xiien, namxiian, hoiliug)
                    dismiss()
                }
                .keyboardShortcut(.defaultAction)
                .disabled(!hasAtLeastOneReading)
            }
        }
        .padding(20)
        .frame(width: 400)
    }

    private var hasAtLeastOneReading: Bool {
        let x = xiien.trimmingCharacters(in: .whitespacesAndNewlines)
        let n = namxiian.trimmingCharacters(in: .whitespacesAndNewlines)
        let h = hoiliug.trimmingCharacters(in: .whitespacesAndNewlines)
        return !x.isEmpty || !n.isEmpty || !h.isEmpty
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
    }

    private func lookupUniqueReading(hanji: String, lect: UInt32) -> String? {
        var toggles = Taigi_Engine_DictionarySourceToggles()
        toggles.lect = lect
        toggles.kautian = true

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
