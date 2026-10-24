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

    // Hakfa fork (Obs IME data-ui D1): only the sources that exist in this keyboard are listed.
    // The upstream Taigi dictionaries (iTaigi, plant names, Taiwanese-Japanese, Tai-Hua, STTI,
    // the accent and appendix sub-options, Khiin / variant / LKK characters, the supplement
    // word list) have no Hakka data, so a switch for them would do nothing. Their stored values
    // stay as they were and the engine still reads them; they simply cannot be changed here.
    // The per-lect sub-options for the MOE dictionary and the certification list are D2.
    var body: some View {
        Form {
            Section {
                Toggle(language.string(.commonMoeDict), isOn: $isKautianEnabled)

                // hakfa: bit 1 (taigitv) is the certification word lists, locked on;
                // desktop-core `SettingsDocument::dictionary_sources` forces it on for the engine too.
                Toggle(language.string(.commonNewwordDict), isOn: .constant(true))
                    .disabled(true)

                // hakfa: bit 6 (kungge) is the HakSpring character supplement, locked on;
                // desktop-core `SettingsDocument::dictionary_sources` forces it on for the engine too.
                Toggle(language.string(.commonKunggeDict), isOn: .constant(true))
                    .disabled(true)
            } header: {
                Text(language.string(.dictionaryMoeSectionTitle))
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

}
