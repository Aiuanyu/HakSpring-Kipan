// Hakfa fork: Spotlight-style quick lookup panel (⌃⌘K).
// Allows looking up Hakfa headwords and synonyms in any app,
// and pasting the selected word or reading directly back to the caret.

import AppKit
import SwiftUI

private final class KeyInterceptingPanel: NSPanel {
    var onEscape: (() -> Void)?

    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { true }

    override func sendEvent(_ event: NSEvent) {
        if event.type == .keyDown && event.keyCode == 53 { // Escape
            onEscape?()
            return
        }
        super.sendEvent(event)
    }
}

@MainActor
public final class QuickLookupPanel: NSObject {
    public static let shared = QuickLookupPanel()

    private var panel: KeyInterceptingPanel?
    private var targetApp: NSRunningApplication?

    public var isShowing: Bool {
        panel?.isVisible == true
    }

    public func toggle(targetApp: NSRunningApplication? = nil) {
        if isShowing {
            hide()
        } else {
            show(targetApp: targetApp)
        }
    }

    public func show(targetApp: NSRunningApplication? = nil) {
        self.targetApp = targetApp ?? NSWorkspace.shared.frontmostApplication
        let p = panel ?? makePanel()
        self.panel = p

        // Position panel at top center of active screen
        if let screen = NSScreen.main {
            let screenRect = screen.visibleFrame
            let x = screenRect.midX - p.frame.width / 2
            let y = screenRect.maxY - p.frame.height - 100
            p.setFrameOrigin(NSPoint(x: x, y: y))
        }

        NSApp.activate()
        p.makeKeyAndOrderFront(nil)
        p.orderFrontRegardless()
    }

    public func hide() {
        panel?.orderOut(nil)
        targetApp?.activate()
    }

    public func commit(text: String) {
        let app = targetApp
        panel?.orderOut(nil)

        // 1. Write to general pasteboard
        let pb = NSPasteboard.general
        pb.clearContents()
        pb.setString(text, forType: .string)

        // 2. Reactivate previous host app
        app?.activate()

        // 3. Simulate Cmd+V paste after brief delay
        DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(150)) {
            let src = CGEventSource(stateID: .combinedSessionState)
            let vKeyCode: CGKeyCode = 0x09 // 'v'
            if let keyDown = CGEvent(keyboardEventSource: src, virtualKey: vKeyCode, keyDown: true) {
                keyDown.flags = .maskCommand
                keyDown.post(tap: .cghidEventTap)
            }
            if let keyUp = CGEvent(keyboardEventSource: src, virtualKey: vKeyCode, keyDown: false) {
                keyUp.flags = .maskCommand
                keyUp.post(tap: .cghidEventTap)
            }
        }
    }

    private func makePanel() -> KeyInterceptingPanel {
        let p = KeyInterceptingPanel(
            contentRect: NSRect(x: 0, y: 0, width: 560, height: 460),
            styleMask: [.titled, .fullSizeContentView, .hudWindow],
            backing: .buffered,
            defer: false
        )
        p.titleVisibility = .hidden
        p.titlebarAppearsTransparent = true
        p.isMovableByWindowBackground = true
        p.level = .floating
        p.hasShadow = true
        p.backgroundColor = .clear

        p.onEscape = { [weak self] in
            self?.hide()
        }

        let hosting = NSHostingController(
            rootView: QuickLookupView(
                onCommit: { [weak self] text in self?.commit(text: text) },
                onDismiss: { [weak self] in self?.hide() }
            )
        )
        p.contentViewController = hosting
        return p
    }
}

// MARK: - SwiftUI View

private let LECT_CHIPS: [(id: String?, name: String)] = [
    (nil, "全部"),
    ("xiien", "四縣"),
    ("namxiian", "南四縣"),
    ("hoiliug", "海陸"),
    ("taibu", "大埔"),
    ("ngiaupin", "饒平"),
    ("zhioon", "詔安"),
]

private struct QuickLookupView: View {
    let onCommit: (String) -> Void
    let onDismiss: () -> Void

    @State private var query = ""
    @State private var selectedLect: String? = nil
    @State private var selectedIndex = 0

    private var results: [GlossEntry] {
        let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.isEmpty { return [] }
        return GlossRepository.shared.search(query: trimmed, lectFilter: selectedLect)
    }

    var body: some View {
        VStack(spacing: 0) {
            // Header: Search input
            HStack(spacing: 10) {
                Image(systemName: "magnifyingglass")
                    .font(.system(size: 18, weight: .medium))
                    .foregroundColor(.secondary)

                TextField("輸入客語漢字、讀音或華語反查（如：飯包、便當、卵、蛋）...", text: $query)
                    .textFieldStyle(.plain)
                    .font(.system(size: 16))
                    .onSubmit {
                        if !results.isEmpty {
                            let idx = selectedIndex.clamped(to: 0..<(results.count))
                            onCommit(results[idx].word)
                        }
                    }

                if !query.isEmpty {
                    Button {
                        query = ""
                    } label: {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(.secondary)
                    }
                    .buttonStyle(.plain)
                }

                Button {
                    onDismiss()
                } label: {
                    Image(systemName: "xmark")
                        .font(.system(size: 12, weight: .bold))
                        .foregroundColor(.secondary)
                        .padding(6)
                        .background(Circle().fill(Color.secondary.opacity(0.15)))
                }
                .buttonStyle(.plain)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 14)
            .background(Color(NSColor.controlBackgroundColor))

            Divider()

            // Lect Filter Chips: "四南海大平安"
            ScrollView(.horizontal, showsIndicators: false) {
                HStack(spacing: 6) {
                    ForEach(LECT_CHIPS, id: \.name) { chip in
                        let isSelected = selectedLect == chip.id
                        Button {
                            selectedLect = chip.id
                            selectedIndex = 0
                        } label: {
                            Text(chip.name)
                                .font(.system(size: 12, weight: isSelected ? .bold : .regular))
                                .padding(.horizontal, 10)
                                .padding(.vertical, 4)
                                .background(
                                    Capsule().fill(isSelected ? Color.accentColor : Color.secondary.opacity(0.12))
                                )
                                .foregroundColor(isSelected ? .white : .primary)
                        }
                        .buttonStyle(.plain)
                    }
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 8)
            }
            .background(Color(NSColor.windowBackgroundColor).opacity(0.5))

            Divider()

            // Results List
            if results.isEmpty {
                VStack(spacing: 12) {
                    Spacer()
                    if query.isEmpty {
                        Image(systemName: "character.book.closed")
                            .font(.system(size: 36))
                            .foregroundColor(.secondary.opacity(0.6))
                        Text("輸入漢字或華語詞彙反查客語詞目")
                            .font(.system(size: 14))
                            .foregroundColor(.secondary)

                        HStack(spacing: 8) {
                            Text("推薦搜尋：")
                                .font(.system(size: 12))
                                .foregroundColor(.secondary)
                            ForEach(["便當", "飯包", "害喜", "春", "卵"], id: \.self) { sample in
                                Button(sample) {
                                    query = sample
                                }
                                .font(.system(size: 12))
                                .buttonStyle(.link)
                            }
                        }
                    } else {
                        Image(systemName: "magnifyingglass")
                            .font(.system(size: 32))
                            .foregroundColor(.secondary.opacity(0.5))
                        Text("無符合「\(query)」的客語詞目或同義詞")
                            .font(.system(size: 14))
                            .foregroundColor(.secondary)
                    }
                    Spacer()
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollViewReader { proxy in
                    List(results.indices, id: \.self) { idx in
                        let entry = results[idx]
                        let isSelected = idx == selectedIndex

                        ResultRowView(
                            entry: entry,
                            isSelected: isSelected,
                            onCommitWord: { onCommit(entry.word) },
                            onCommitReading: { onCommit(entry.reading) },
                            onCommitBoth: { onCommit("\(entry.word) [\(entry.reading)]") }
                        )
                        .id(idx)
                        .listRowInsets(EdgeInsets(top: 4, leading: 10, bottom: 4, trailing: 10))
                        .listRowBackground(
                            RoundedRectangle(cornerRadius: 8)
                                .fill(isSelected ? Color.accentColor.opacity(0.15) : Color.clear)
                        )
                        .onTapGesture {
                            selectedIndex = idx
                            onCommit(entry.word)
                        }
                    }
                    .listStyle(.plain)
                    .onChange(of: selectedIndex) { _, newIdx in
                        proxy.scrollTo(newIdx, anchor: .center)
                    }
                }
            }

            Divider()

            // Footer Tips
            HStack {
                HStack(spacing: 12) {
                    TipItem(key: "Enter", desc: "送出字")
                    TipItem(key: "⌥Enter", desc: "送出讀音")
                    TipItem(key: "Esc", desc: "關閉")
                }
                Spacer()
                Text("客源翠隨身查詞")
                    .font(.system(size: 11))
                    .foregroundColor(.secondary.opacity(0.8))
            }
            .padding(.horizontal, 14)
            .padding(.vertical, 8)
            .background(Color(NSColor.windowBackgroundColor))
        }
        .frame(width: 560, height: 460)
        .background(VisualEffectBackground())
        .clipShape(RoundedRectangle(cornerRadius: 12))
    }
}

private struct ResultRowView: View {
    let entry: GlossEntry
    let isSelected: Bool
    let onCommitWord: () -> Void
    let onCommitReading: () -> Void
    let onCommitBoth: () -> Void

    var body: some View {
        HStack(alignment: .top, spacing: 10) {
            VStack(alignment: .leading, spacing: 4) {
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text(entry.word)
                        .font(.system(size: 17, weight: .bold))
                        .foregroundColor(.primary)

                    Text(entry.reading)
                        .font(.system(size: 14, weight: .medium, design: .serif))
                        .foregroundColor(.secondary)

                    Text(entry.lectName)
                        .font(.system(size: 11))
                        .padding(.horizontal, 5)
                        .padding(.vertical, 1)
                        .background(Capsule().fill(Color.secondary.opacity(0.15)))
                        .foregroundColor(.secondary)

                    if entry.isSynonymHit, let kw = entry.matchedKeyword {
                        Text("同義：\(kw)")
                            .font(.system(size: 10, weight: .medium))
                            .padding(.horizontal, 5)
                            .padding(.vertical, 1)
                            .background(Capsule().fill(Color.teal.opacity(0.2)))
                            .foregroundColor(.teal)
                    }

                    if !entry.dialectTags.isEmpty {
                        Text(entry.dialectTags)
                            .font(.system(size: 10))
                            .foregroundColor(.orange)
                    }
                }

                if !entry.gloss.isEmpty {
                    Text(entry.gloss)
                        .font(.system(size: 12))
                        .foregroundColor(.secondary)
                        .lineLimit(2)
                }
            }

            Spacer()

            // Web Audio Link Button
            Button {
                openHakSpringWeb(word: entry.word, lectId: entry.lectId)
            } label: {
                HStack(spacing: 3) {
                    Image(systemName: "globe")
                        .font(.system(size: 10))
                    Text("客源翠")
                        .font(.system(size: 11))
                }
                .padding(.horizontal, 7)
                .padding(.vertical, 3)
                .background(RoundedRectangle(cornerRadius: 6).stroke(Color.accentColor.opacity(0.5), lineWidth: 1))
                .foregroundColor(.accentColor)
            }
            .buttonStyle(.plain)
            .help("在客源翠開啟聽音檔與查看例句")
        }
        .padding(.vertical, 6)
        .contentShape(Rectangle())
    }

    private func openHakSpringWeb(word: String, lectId: String) {
        let kiongParam: String
        switch lectId {
        case "xiien": kiongParam = "si"
        case "namxiian": kiongParam = "na"
        case "hoiliug": kiongParam = "ha"
        case "taibu": kiongParam = "da"
        case "ngiaupin": kiongParam = "rh"
        case "zhioon": kiongParam = "zh"
        default: kiongParam = "si"
        }
        let encoded = word.addingPercentEncoding(withAllowedCharacters: .urlQueryAllowed) ?? word
        let urlString = "https://hakspring.pages.dev/?musiid=hak&ca=\(encoded)&kiong=\(kiongParam)"
        if let url = URL(string: urlString) {
            NSWorkspace.shared.open(url)
        }
    }
}

private struct TipItem: View {
    let key: String
    let desc: String

    var body: some View {
        HStack(spacing: 4) {
            Text(key)
                .font(.system(size: 10, weight: .bold, design: .monospaced))
                .padding(.horizontal, 4)
                .padding(.vertical, 1)
                .background(RoundedRectangle(cornerRadius: 3).fill(Color.secondary.opacity(0.18)))
            Text(desc)
                .font(.system(size: 11))
                .foregroundColor(.secondary)
        }
    }
}

private struct VisualEffectBackground: NSViewRepresentable {
    func makeNSView(context: Context) -> NSVisualEffectView {
        let view = NSVisualEffectView()
        view.material = .hudWindow
        view.blendingMode = .behindWindow
        view.state = .active
        return view
    }

    func updateNSView(_ nsView: NSVisualEffectView, context: Context) {}
}

private extension Comparable {
    func clamped(to limits: Range<Self>) -> Self {
        min(max(self, limits.lowerBound), limits.upperBound)
    }
}
