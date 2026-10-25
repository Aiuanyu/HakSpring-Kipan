// Which dictionaries the keyboard draws candidates from.

import SwiftUI

/// The dictionary source toggles, in the sections iOS groups them into
/// (`ios/.../App/Tabs/Dictionary/DictionaryTab.swift`).
///
/// `@AppStorage` per toggle rather than one snapshot object: each row is an
/// independent setting the engine live-reads, and binding them individually is
/// what makes a `defaults write` — or a future settings import — show up in the
/// form without anything having to be told.
struct DictionaryTogglesView: View {
    @Environment(DisplayLanguageStore.self) private var language

    @AppStorage(SettingsStore.Keys.isKautianEnabled.name)
    private var isKautianEnabled = SettingsStore.Keys.isKautianEnabled.defaultValue

    // Hakfa fork (Obs IME data-ui D2): which lects can be typed in. One switch per lect under the
    // MOE dictionary; the certification list's rows below it follow them and cannot be changed.
    @AppStorage(SettingsStore.Keys.lectEnabled(.xiien).name)
    private var isXiienEnabled = SettingsStore.Keys.lectEnabled(.xiien).defaultValue
    @AppStorage(SettingsStore.Keys.lectEnabled(.namxiian).name)
    private var isNamxiianEnabled = SettingsStore.Keys.lectEnabled(.namxiian).defaultValue
    @AppStorage(SettingsStore.Keys.lectEnabled(.hoiliug).name)
    private var isHoiliugEnabled = SettingsStore.Keys.lectEnabled(.hoiliug).defaultValue

    @AppStorage(SettingsStore.Keys.isMeinongEnabled.name)
    private var isMeinongEnabled = SettingsStore.Keys.isMeinongEnabled.defaultValue

    // Hakfa fork (Obs IME data-ui D1): only the sources that exist in this keyboard are listed.
    // The upstream Taigi dictionaries (iTaigi, plant names, Taiwanese-Japanese, Tai-Hua, STTI,
    // the accent and appendix sub-options, Khiin / variant / LKK characters, the supplement
    // word list) have no Hakka data, so a switch for them would do nothing. Their stored values
    // stay as they were and the engine still reads them; they simply cannot be changed here.
    var body: some View {
        Form {
            Section {
                Toggle(language.string(.commonMoeDict), isOn: $isKautianEnabled)
                Group {
                    ForEach(Lect.allCases, id: \.self) { lect in
                        Toggle(lect.displayName, isOn: enabledBinding(lect))
                            // The last lect on stays on: the keyboard must have one to type in.
                            .disabled(isOn(lect) && enabledCount == 1)
                        // Meinong is a switch under 南四縣, not a lect of its own (USER 2026-10-25).
                        if lect == .namxiian {
                            Toggle("美濃音 (Minung)", isOn: $isMeinongEnabled)
                                .padding(.leading, Metrics.subcollectionIndent)
                                .disabled(!isNamxiianEnabled)
                        }
                    }
                }
                .padding(.leading, Metrics.subcollectionIndent)
                // Disabled, not cleared: the master switch says whether this
                // dictionary is searched at all, and turning it back on has to
                // return the lects the user had chosen rather than all of them.
                .disabled(!isKautianEnabled)

                // hakfa: bit 1 (taigitv) is the certification word lists, locked on;
                // desktop-core `SettingsDocument::dictionary_sources` forces it on for the engine too.
                Toggle(language.string(.commonNewwordDict), isOn: .constant(true))
                    .disabled(true)
                // The certification list has five lects (no 南四縣 of its own): its 四縣 follows
                // the MOE dictionary's 四縣 or 南四縣, the others follow the same lect.
                Group {
                    Toggle(Lect.xiien.displayName, isOn: .constant(isOn(.xiien) || isOn(.namxiian)))
                    Toggle(Lect.hoiliug.displayName, isOn: .constant(isOn(.hoiliug)))
                }
                .padding(.leading, Metrics.subcollectionIndent)
                .disabled(true)

                // hakfa: bit 6 (kungge) is the HakSpring character supplement, locked on;
                // desktop-core `SettingsDocument::dictionary_sources` forces it on for the engine too.
                Toggle(language.string(.commonKunggeDict), isOn: .constant(true))
                    .disabled(true)
            } header: {
                Text(language.string(.dictionaryMoeSectionTitle))
            } footer: {
                Text("客語能力認證詞彙跟著教育部辭典：辭典開了四縣或南四縣，認證的四縣就開；海陸等其他腔同款。")
            }

            // Its own section, at the end, drawn the way the Shortcuts and Appearance
            // panes draw theirs: it acts on every toggle above it rather than
            // on any one of them.
            Section {
                WideActionRow(titleKey: .themeEditorResetAll, action: restoreDefaults)
            }
        }
        .formStyle(.grouped)
    }

    /// Puts every source and accent toggle back to what a fresh install searches.
    ///
    /// The `@AppStorage` bindings above repaint on their own: removing a key is
    /// a `UserDefaults` change like any other, and each binding falls back to
    /// the default it was declared with.
    private func restoreDefaults() {
        SettingsStore().resetDictionarySources()
    }

    private func isOn(_ lect: Lect) -> Bool {
        switch lect {
        case .xiien: isXiienEnabled
        case .namxiian: isNamxiianEnabled
        case .hoiliug: isHoiliugEnabled
        }
    }

    private var enabledCount: Int {
        Lect.allCases.filter(isOn).count
    }

    private func enabledBinding(_ lect: Lect) -> Binding<Bool> {
        switch lect {
        case .xiien: $isXiienEnabled
        case .namxiian: $isNamxiianEnabled
        case .hoiliug: $isHoiliugEnabled
        }
    }

    private enum Metrics {
        static let subcollectionIndent: CGFloat = 16
    }
}
