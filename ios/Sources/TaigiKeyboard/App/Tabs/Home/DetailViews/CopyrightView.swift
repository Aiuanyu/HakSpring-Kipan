import SwiftUI

// Standardized short license names — language-invariant, never translated (kept as constants rather
// than i18n keys per the multi-language plan). Mirrors Android content/CopyrightData.kt.
private enum License {
    static let silOpenFont = "SIL Open Font License"
    static let silOpenFont11 = "SIL Open Font License 1.1"
    static let ccByNd3Tw = "CC BY-ND 3.0 TW"
    static let ogdlTaiwan10 = "OGDL-Taiwan-1.0"
    static let apache2 = "Apache License 2.0"
}

/// Copyright notices for fonts, dictionaries, and open-source projects.
struct CopyrightView: View {
    @Environment(DisplayLanguageStore.self) private var lang
    var body: some View {
        Form {
            // Maruko Gothic (丸仔烏體)
            CopyrightSection(
                title: lang.string(.commonFontMarukoGothic),
                description: "© The Maruko Gothic Authors (syimyuz) / Carlito © Google Inc.",
                license: License.silOpenFont11,
                licenseURL: "https://openfontlicense.org/",
                websiteURL: "https://github.com/syimyuz/maruko-gothic",
            )

            // Iansui (芫荽體)
            CopyrightSection(
                title: lang.string(.commonFontIansui),
                description: lang.string(.homeButTaiwanCopyright),
                license: License.silOpenFont11,
                licenseURL: "https://openfontlicense.org/",
                websiteURL: "https://github.com/ButTaiwan/iansui",
            )

            // Open Huninn (粉圓體)
            CopyrightSection(
                title: lang.string(.commonFontOpenHuninn),
                description: lang.string(.homeOpenFontCopyright),
                license: License.silOpenFont,
                licenseURL: "https://openfontlicense.org/",
                websiteURL: "https://justfont.com/huninn/",
            )

            // GenYoMin (源樣明體)
            CopyrightSection(
                title: lang.string(.commonFontGenYoMin),
                description: lang.string(.homeButTaiwanCopyright),
                license: License.silOpenFont11,
                licenseURL: "https://openfontlicense.org/",
                websiteURL: "https://github.com/ButTaiwan/genyo-font",
            )

            // GenYoGothic (源樣烏體)
            CopyrightSection(
                title: lang.string(.commonFontGenYoGothic),
                description: lang.string(.homeButTaiwanCopyright),
                license: License.silOpenFont11,
                licenseURL: "https://openfontlicense.org/",
                websiteURL: "https://github.com/ButTaiwan/genyog-font",
            )

            // MOE Hakfa Dictionary (教育部臺灣客語辭典)
            CopyrightSection(
                title: lang.string(.commonMoeDict),
                description: lang.string(.homeMoeCopyright),
                license: License.ccByNd3Tw,
                licenseURL: "https://creativecommons.org/licenses/by-nd/3.0/tw/",
                websiteURL: "https://hakkadict.moe.edu.tw/",
            )

            // Hakfa Proficiency Certification Vocabulary (客語能力認證詞彙)
            CopyrightSection(
                title: lang.string(.commonNewwordDict),
                description: lang.string(.homeNewwordCopyright),
                license: License.ogdlTaiwan10,
                licenseURL: "https://spdx.org/licenses/OGDL-Taiwan-1.0.html",
                websiteURL: "https://elearning.hakka.gov.tw/hakka/download-files",
            )

            // Upstream Software (TaigiKeyboard)
            CopyrightSection(
                title: "TaigiKeyboard",
                description: "© 2025-2026 蘇民弦（Soo Bîn-hiân）",
                license: License.apache2,
                licenseURL: "https://www.apache.org/licenses/LICENSE-2.0",
                websiteURL: "https://github.com/taigikeyboard/taigikeyboard",
            )
        }
        .navigationTitle(lang.string(.homeCopyrightNotice))
        .navigationBarTitleDisplayMode(.large)
    }
}

// MARK: - Copyright Section

private struct CopyrightSection: View {
    @Environment(DisplayLanguageStore.self) private var lang
    let title: String
    let description: String
    let license: String
    let licenseURL: String
    let websiteURL: String?

    var body: some View {
        Section {
            VStack(alignment: .leading, spacing: 8) {
                Text(title)
                    .font(AppStyle.headlineFont)

                Text(description)
                    .foregroundColor(.secondary)

                Text(license)
                    .font(AppStyle.captionFont)
                    .foregroundColor(.secondary)
                    .italic()
            }

            Link(destination: URL(string: licenseURL)!) {
                Label(lang.string(.homeViewLicense), systemImage: "doc.text")
            }

            if let websiteURL {
                Link(destination: URL(string: websiteURL)!) {
                    Label(lang.string(.commonViewWebsite), systemImage: "globe")
                }
            }
        }
    }
}
