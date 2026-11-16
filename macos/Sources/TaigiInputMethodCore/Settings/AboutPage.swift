// The About page: what the project is and where to find it.

import SwiftUI

/// What the input-source menu's About row opens (USER 2026-09-20): two lines the
/// USER wrote; the sponsor and the seven community links as rows; the
/// attribution line.
///
/// A grouped `Form` like every other pane (USER 2026-09-20: "make it a page"), so the
/// page sits where the settings do and reads in the same cards. No app icon, no
/// name and no version — the update row on General already says which build this is.
struct AboutPage: View {
    @Environment(DisplayLanguageStore.self) private var language
    @State private var didCopyDiagnostic = false

    private var currentLectName: String {
        SettingsStore().effectiveLect.displayName
    }

    private var diagnosticPayload: String {
        let appVer = AppVersion.installed
        let lexVer = LexiconVersion.current.displaySummary
        let lect = currentLectName
        let osVer = ProcessInfo.processInfo.operatingSystemVersionString
        return """
        客源翠企盤 HakSpring-Kipan
        App: v\(appVer)
        詞庫版本: \(lexVer)
        目前腔頭: \(lect)
        OS: macOS \(osVer)
        """.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    var body: some View {
        Form {
            // 版本與系統狀態 (Version & Status)
            Section {
                LabeledContent(language.string(.commonAppVersion)) {
                    Text(AppVersion.installed)
                        .foregroundStyle(.secondary)
                }
                LabeledContent(language.string(.commonLexiconVersion)) {
                    Text(LexiconVersion.current.displaySummary)
                        .foregroundStyle(.secondary)
                }
                LabeledContent(language.string(.commonActiveLect)) {
                    Text(currentLectName)
                        .foregroundStyle(.secondary)
                }
                HStack {
                    Button {
                        let pasteboard = NSPasteboard.general
                        pasteboard.clearContents()
                        pasteboard.setString(diagnosticPayload, forType: .string)
                        didCopyDiagnostic = true
                    } label: {
                        Label(
                            didCopyDiagnostic
                                ? language.string(.commonDiagnosticCopiedNotice)
                                : language.string(.commonDiagnosticCopyNotice),
                            systemImage: didCopyDiagnostic ? "checkmark" : "doc.on.doc"
                        )
                    }

                    Spacer()

                    if let url = URL(string: "https://hakspring.pages.dev/kipan/") {
                        Link(destination: url) {
                            Label(language.string(.commonCheckUpdateAction), systemImage: "arrow.triangle.2.circlepath")
                        }
                    }
                }
                .padding(.top, 2)
            }

            // No heading (USER 2026-09-20: "no 'TaigiKeyboard' heading needed"): the window
            // title already names the page. Leading-aligned like the rows
            // under it (USER 2026-09-20: "left-align the copy").
            Section {
                VStack(alignment: .leading, spacing: Metrics.paragraphSpacing) {
                    Text(language.string(.homeAboutIntroProject))
                    Text(language.string(.homeAboutIntroMaintainer))
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.vertical, Metrics.cardInset)
            }

            // Every action a row, the sponsor link first (USER 2026-09-20: no
            // lone button on the page).
            Section {
                // Hakfa fork: hide upstream ECPay sponsor link during beta.
                // ExternalLinkButton(titleKey: .desktopSponsorLink, url: Self.sponsorURL, style: .row(.heart))
                ExternalLinkButton(titleKey: .homeWebsiteLink, url: Self.websiteURL, style: .row(.globe))
                ExternalLinkButton(titleKey: .homeGithubLink, url: Self.githubURL, style: .row(.github))
                ExternalLinkButton(titleKey: .homeDiscordLink, url: Self.discordURL, style: .row(.discord))
                ExternalLinkButton(titleKey: .homeFacebookLink, url: Self.facebookURL, style: .row(.facebook))
                ExternalLinkButton(titleKey: .homeInstagramLink, url: Self.instagramURL, style: .row(.instagram))
                ExternalLinkButton(titleKey: .homeThreadsLink, url: Self.threadsURL, style: .row(.threads))
                ExternalLinkButton(titleKey: .homeEmailLink, url: Self.emailURL, style: .row(.envelope))
            }

            // 致謝與授權條款 (Acknowledgements & Licenses)
            Section {
                LicenseRow(
                    title: "TaigiKeyboard",
                    subtitle: "© 2025-2026 蘇民弦（Soo Bîn-hiân）",
                    license: "Apache-2.0",
                    licenseURL: "https://www.apache.org/licenses/LICENSE-2.0",
                    websiteURL: "https://github.com/taigikeyboard/taigikeyboard"
                )
                LicenseRow(
                    title: language.string(.commonMoeDict),
                    subtitle: "© 中華民國教育部",
                    license: "CC BY-ND 3.0 TW",
                    licenseURL: "https://creativecommons.org/licenses/by-nd/3.0/tw/",
                    websiteURL: "https://hakkadict.moe.edu.tw/"
                )
                LicenseRow(
                    title: language.string(.commonNewwordDict),
                    subtitle: "© 客家委員會",
                    license: "OGDL-Taiwan-1.0",
                    licenseURL: "https://spdx.org/licenses/OGDL-Taiwan-1.0.html",
                    websiteURL: "https://elearning.hakka.gov.tw/hakka/download-files"
                )
                LicenseRow(
                    title: language.string(.commonFontMarukoGothic),
                    subtitle: "© The Maruko Gothic Authors (syimyuz) / Carlito © Google Inc.",
                    license: "SIL OFL 1.1",
                    licenseURL: "https://openfontlicense.org/",
                    websiteURL: "https://github.com/syimyuz/maruko-gothic"
                )
                LicenseRow(
                    title: language.string(.commonFontIansui),
                    subtitle: "© ButTaiwan (But Ko / Fontworks Inc.)",
                    license: "SIL OFL 1.1",
                    licenseURL: "https://openfontlicense.org/",
                    websiteURL: "https://github.com/ButTaiwan/iansui"
                )
                LicenseRow(
                    title: language.string(.commonFontOpenHuninn),
                    subtitle: "© justfont",
                    license: "SIL OFL 1.1",
                    licenseURL: "https://openfontlicense.org/",
                    websiteURL: "https://justfont.com/huninn/"
                )
                LicenseRow(
                    title: language.string(.commonFontGenYoMin),
                    subtitle: "© ButTaiwan (Source Han Serif)",
                    license: "SIL OFL 1.1",
                    licenseURL: "https://openfontlicense.org/",
                    websiteURL: "https://github.com/ButTaiwan/genyo-font"
                )
                LicenseRow(
                    title: language.string(.commonFontGenYoGothic),
                    subtitle: "© ButTaiwan (Source Han Sans)",
                    license: "SIL OFL 1.1",
                    licenseURL: "https://openfontlicense.org/",
                    websiteURL: "https://github.com/ButTaiwan/genyog-font"
                )
            } header: {
                Text("致謝摎授權條款")
            } footer: {
                Text(language.string(.homeCopyrightLine))
                    .frame(maxWidth: .infinity)
                    .padding(.top, Metrics.footerGap)
            }
        }
        .formStyle(.grouped)
    }

    private static let websiteURL = URL(string: "https://hakspring.pages.dev/kipan/")
    private static let githubURL = URL(string: "https://github.com/Aiuanyu/HakSpring-Kipan")
    private static let discordURL = URL(string: "https://www.reddit.com/r/HakSpring")
    private static let facebookURL = URL(string: "https://www.facebook.com/Aiuanyu/")
    private static let instagramURL = URL(string: "https://www.instagram.com/aiuanyu/")
    private static let threadsURL = URL(string: "https://www.threads.net/@aiuanyu")
    private static let emailURL = URL(string: "mailto:aiuanyu777@gmail.com")
    private static let sponsorURL = URL(string: "https://p.ecpay.com.tw/AA663DE")

    private enum Metrics {
        /// Between paragraphs of one text.
        static let paragraphSpacing: CGFloat = 10

        /// Air above and below a card of running text; a row of one line needs none.
        static let cardInset: CGFloat = 4

        /// Between the last card and the attribution under it.
        static let footerGap: CGFloat = 8
    }
}

private struct LicenseRow: View {
    let title: String
    let subtitle: String
    let license: String
    let licenseURL: String?
    let websiteURL: String?

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(title)
                    .font(.body.weight(.medium))
                Spacer()
                Text(license)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(Color.secondary.opacity(0.12), in: RoundedRectangle(cornerRadius: 4))
            }
            Text(subtitle)
                .font(.caption)
                .foregroundStyle(.secondary)
            HStack(spacing: 16) {
                if let licenseURL, let url = URL(string: licenseURL) {
                    Link(destination: url) {
                        Label("授權條款", systemImage: "doc.text")
                            .font(.caption)
                    }
                }
                if let websiteURL, let url = URL(string: websiteURL) {
                    Link(destination: url) {
                        Label("官方網站", systemImage: "globe")
                            .font(.caption)
                    }
                }
            }
            .padding(.top, 2)
        }
        .padding(.vertical, 4)
    }
}
