@testable import TaigiInputMethodCore
import XCTest

/// The display-language roster and the Automatic policy. Every case here pins a contract macOS
/// shares with iOS and Android — see `docs/architecture/behavioral-invariants.md` §37 — so a failure
/// means the three platforms have drifted, not that macOS alone is wrong.
final class DisplayLanguageTests: XCTestCase {
    func testINVARIANT_DISPLAY_LANGUAGE_PRODUCTION_ROSTER_pinsHakhaTaihaEnglishJapaneseTailoAndPoj() {
        // Must equal the iOS/Android productionLanguages rosters and tools/i18n PRODUCTION_LANGUAGES,
        // same order: the generator gates translation completeness on that list.
        XCTAssertEqual(DisplayLanguage.productionLanguages.map(\.tag), ["hakha", "taiha", "en", "ja", "tailo", "poj"])
    }

    func testINVARIANT_DISPLAY_LANGUAGE_PRODUCTION_ROSTER_excludesSystemSelectionPolicy() {
        // SYSTEM is a selection policy with no authored strings; it must never join the roster the
        // completeness gate walks.
        XCTAssertFalse(DisplayLanguage.productionLanguages.contains(.system))
    }

    func testINVARIANT_DISPLAY_LANGUAGE_PRODUCTION_ROSTER_selectableLeadsWithSystem() {
        XCTAssertEqual(
            DisplayLanguage.selectableLanguages.map(\.tag),
            ["system", "hakha", "taiha", "en", "ja", "tailo", "poj"],
        )
    }

    func testFromTag_everySelectableTagRoundTrips() {
        for language in DisplayLanguage.selectableLanguages {
            XCTAssertEqual(DisplayLanguage.fromTag(language.tag), language, "tag \(language.tag)")
        }
    }

    func testFromTag_unknownTagFallsBackToHakha() {
        XCTAssertEqual(DisplayLanguage.fromTag("xx"), .hakha)
        XCTAssertEqual(DisplayLanguage.fromTag(""), .hakha)
    }

    func testDefaultTag_resolvesToSystemAutomatic() {
        // A fresh install follows the device OS locale rather than pinning Hakha.
        XCTAssertEqual(DisplayLanguage.defaultTag, "system")
        XCTAssertEqual(DisplayLanguage.fromTag(DisplayLanguage.defaultTag), .system)
    }

    func testINVARIANT_DISPLAY_LANGUAGE_AUTOMATIC_RESOLUTION_mapsDeviceSubtagToAuthoredLanguage() {
        // Japanese device → Japanese; English → English; everything else (Chinese included, or a
        // device that reports nothing) → Hakha.
        XCTAssertEqual(DisplayLanguage.resolveAutomatic("ja"), .japanese)
        XCTAssertEqual(DisplayLanguage.resolveAutomatic("en"), .english)
        XCTAssertEqual(DisplayLanguage.resolveAutomatic("zh"), .hakha)
        XCTAssertEqual(DisplayLanguage.resolveAutomatic("fr"), .hakha)
        XCTAssertEqual(DisplayLanguage.resolveAutomatic(""), .hakha)
    }

    func testINVARIANT_DISPLAY_LANGUAGE_AUTOMATIC_RESOLUTION_effectiveLanguageOnlyResolvesSystem() {
        // An explicitly picked language is itself no matter what the device reports; only SYSTEM follows it.
        XCTAssertEqual(DisplayLanguage.hakha.effectiveLanguage("ja"), .hakha)
        XCTAssertEqual(DisplayLanguage.taiha.effectiveLanguage("ja"), .taiha)
        XCTAssertEqual(DisplayLanguage.english.effectiveLanguage("ja"), .english)
        XCTAssertEqual(DisplayLanguage.system.effectiveLanguage("ja"), .japanese)
        XCTAssertEqual(DisplayLanguage.system.effectiveLanguage("en"), .english)
        XCTAssertEqual(DisplayLanguage.system.effectiveLanguage("zh"), .hakha)
    }

    func testEndonym_isTheLanguagesOwnName() {
        // Language-invariant strings, so they are not i18n keys — and they MUST match iOS/Android.
        XCTAssertEqual(DisplayLanguage.hakha.endonym, "諸夏字／漢字客文")
        XCTAssertEqual(DisplayLanguage.taiha.endonym, "漢字（臺文）")
        XCTAssertEqual(DisplayLanguage.tailo.endonym, "Tâi-lô")
        XCTAssertEqual(DisplayLanguage.poj.endonym, "Pe̍h-ōe-jī")
        XCTAssertEqual(DisplayLanguage.japanese.endonym, "日本語")
        XCTAssertEqual(DisplayLanguage.english.endonym, "English")
    }
}
