// Hakfa fork: the Annotate in Brackets page — the switch and the template each lead order writes in.

import SwiftUI

/// The page of Annotate in Brackets. One template per lead order — the Hanji leads (`諸夏字 [Lomàsīi]`)
/// or the romanization leads (`Lomàsīi [諸夏字]`) — each with an example of what it writes. The lead
/// itself still follows the Output Script setting. Wording is Hakfa-only (no i18n key), like the other
/// fork-only screens. Bound with `@AppStorage`, so the candidate window and this page read one value.
struct AnnotationSettingsView: View {
    @Environment(DisplayLanguageStore.self) private var language

    @AppStorage(SettingsStore.Keys.isOutputBothScripts.name)
    private var isOutputBothScripts = SettingsStore.Keys.isOutputBothScripts.defaultValue

    @AppStorage(SettingsStore.Keys.annotationTemplateHanjiFirst.name)
    private var hanjiFirst = SettingsStore.Keys.annotationTemplateHanjiFirst.defaultValue

    @AppStorage(SettingsStore.Keys.annotationTemplateRomanFirst.name)
    private var romanFirst = SettingsStore.Keys.annotationTemplateRomanFirst.defaultValue

    @AppStorage(SettingsStore.Keys.candidateDisplayMode.name)
    private var candidateDisplayMode = SettingsStore.Keys.candidateDisplayMode.defaultValue

    var body: some View {
        Form {
            Section {
                Toggle(language.string(.settingsOutputBothScripts), isOn: $isOutputBothScripts)
                    .disabled(!candidateDisplayMode.showsHanji)
            }
            Section {
                templatePicker("漢字在前", selection: $hanjiFirst, lead: "諸夏字", annotation: "Lomàsīi")
                templatePicker("羅馬字在前", selection: $romanFirst, lead: "Lomàsīi", annotation: "諸夏字")
            } footer: {
                Text("漢字抑係羅馬字在前，照「輸出文字」个設定。候選窗个預覽摎送出去个字共樣；ruby 樣板个候選窗淨顯示「ᴿᵀ 讀音」。")
            }
            Section {
                Button("恢復預設") {
                    SettingsStore().resetAnnotationSettings()
                }
            }
        }
        .formStyle(.grouped)
    }

    private func templatePicker(
        _ title: String,
        selection: Binding<BracketTemplate>,
        lead: String,
        annotation: String,
    ) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Picker(title, selection: selection) {
                ForEach(BracketTemplate.allCases, id: \.self) { template in
                    Text(template.exampleLabel).tag(template)
                }
            }
            // The example the choice writes, under the row.
            Text("→ " + selection.wrappedValue.sample(lead: lead, annotation: annotation))
                .font(.callout)
                .foregroundStyle(.secondary)
                .textSelection(.enabled)
        }
    }
}
