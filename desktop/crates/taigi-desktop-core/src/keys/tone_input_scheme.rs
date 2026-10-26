//! Which keys type a tone — and, by the same choice, which keys pick a
//! candidate. macOS keeps a Swift twin for the window labels:
//! `ToneInputScheme.swift`.

use super::slot_key_set::CandidateSlotKeySet;
use crate::settings::Lect;
use crate::settings::SettingChoice;
use crate::strings::StringKey;

/// Which keys type a tone — the half of the key contract that decides, in
/// turn, which keys pick a candidate.
///
/// Standard and Telex are the same eight letters and the same nine digits,
/// swapped (USER 2026-09-08). Under `Standard` a digit is the TL/POJ tone
/// marker (`tai5`), so the letters no syllable spells are free to pick
/// candidates; under `Telex` those letters carry the tones, so the digits
/// are free to pick. Neither half is chosen on its own: the slot key set is
/// derived here (`slot_key_set`) rather than stored, so the two can never
/// disagree about who owns `v` or `3`.
///
/// The letter table is the kahiok scheme (madmaxieee/taigi-telex) minus its
/// `c` → `tsh` key, because POJ spells `ch` / `chh` with `c`. Tone 6 has no
/// free letter and is not offered. The semantics — which tone each key
/// writes, how `z` resolves by input mode, what a repeated key does — live in
/// the engine (`engine/composing/src/telex.rs`); this side only decides which
/// keys reach it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ToneInputScheme {
    /// 方案 3：符號編號（Ctrl+2 3 5 6 7 對應 á à â ǎ ā）
    Standard,
    /// 方案 1：五度調值（Ctrl+24、Ctrl+33 等，照當前腔調動態換算為調號）
    Scale,
    /// Letters type tones; `1`…`9` pick candidates.
    Telex,
}

impl ToneInputScheme {
    /// Lower-case Telex keys: `x v y d w q` for tones 1/4 2/8 3 5 7 9 (`x` and `v` split
    /// their pair by the coda, engine-side), `z` for
    /// the affricate initial (`ts` / `ch`; `zh` then spells `tsh` / `chh`),
    /// `f` for the hyphen. Mirrors the engine's `composing::telex::TELEX_KEYS`
    /// — the two lists must name the same keys, or a key classified as Telex
    /// here would be ignored there. CROSS-PLATFORM INVARIANT — the one list for
    /// macOS, Windows and Linux.
    pub const TELEX_KEYS: &'static str = "vydwxqzf";

    /// The keys that pick a candidate under this scheme.
    pub fn slot_key_set(self) -> CandidateSlotKeySet {
        CandidateSlotKeySet::Digits
    }

    /// The picker row's i18n key. macOS keeps a Swift twin of the picker:
    /// `GeneralSettingsView.swift`.
    pub fn label_key(self) -> StringKey {
        match self {
            Self::Standard => StringKey::SettingsToneSchemeStandard,
            Self::Scale => StringKey::SettingsToneSchemeScale,
            Self::Telex => StringKey::SettingsToneSchemeTelex,
        }
    }

    /// Whether `character` is a Telex key in either case. ASCII only: the
    /// engine reads the key as an ASCII letter, and a `v` from another
    /// script is document text.
    pub fn is_telex_key(character: char) -> bool {
        character.is_ascii() && Self::TELEX_KEYS.contains(character.to_ascii_lowercase())
    }

    /// Whether `character` may start a composition on its own. Only `z` /
    /// `Z`: it types an initial, so it begins a syllable the way any letter
    /// does. A tone letter or `f` has nothing to attach to when idle, and
    /// passes to the host like an idle digit.
    pub fn starts_composition(character: char) -> bool {
        matches!(character, 'z' | 'Z')
    }
}

/// 方案 1：將五度調值字串（如 "24", "33"）依腔調轉為 TL 調號字符 ('2', '3', '5', '6', '7', '1', '4')。
pub fn resolve_scale_tone(lect: Lect, scale_input: &str, is_checked: bool) -> Option<char> {
    if is_checked {
        return match scale_input {
            "2" => Some('3'), // 陰入 (à)
            "5" => Some('4'), // 陽入 (無標)
            _ => None,
        };
    }
    match lect {
        Lect::Xiien | Lect::Namxiian => match scale_input {
            "24" => Some('2'), // 陰平 (á)
            "11" => Some('6'), // 陽平 (ǎ)
            "31" => Some('3'), // 上聲 (à)
            "55" => Some('1'), // 去聲 (無標)
            _ => None,
        },
        Lect::Hoiliug => match scale_input {
            "53" => Some('3'), // 陰平 (à)
            "24" => Some('2'), // 上聲 (á)
            "11" => Some('6'), // 陰去 (ǎ)
            "33" => Some('7'), // 陽去 (ā)
            "55" => Some('1'), // 陽平 (無標)
            _ => None,
        },
    }
}

impl SettingChoice for ToneInputScheme {
    const ALL: &'static [Self] = &[Self::Standard, Self::Scale, Self::Telex];
    const DEFAULT: Self = Self::Standard;
    fn raw(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Scale => "scale",
            Self::Telex => "telex",
        }
    }

    fn from_raw(raw: &str) -> Option<Self> {
        match raw {
            "standard" | "symbol" => Some(Self::Standard),
            "scale" | "contour" => Some(Self::Scale),
            "telex" => Some(Self::Telex),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slot_key_set_is_derived_from_the_scheme() {
        // trace: `slot_key_set` — Standard → `BareKeys`, Telex → `Digits`;
        // `DEFAULT` is Standard.
        assert_eq!(
            ToneInputScheme::Standard.slot_key_set(),
            CandidateSlotKeySet::Digits
        );
        assert_eq!(
            ToneInputScheme::Telex.slot_key_set(),
            CandidateSlotKeySet::Digits
        );
        assert_eq!(ToneInputScheme::DEFAULT, ToneInputScheme::Standard);
    }

    #[test]
    fn the_telex_keys_are_the_eight_letters_in_either_case() {
        for letter in "vydwxqzf".chars() {
            assert!(ToneInputScheme::is_telex_key(letter), "{letter}");
            assert!(
                ToneInputScheme::is_telex_key(letter.to_ascii_uppercase()),
                "{letter}"
            );
        }
        for other in ['a', 't', 'c', '3', '-', ';', ' ', 'ｖ', 'ｚ'] {
            assert!(!ToneInputScheme::is_telex_key(other), "{other}");
        }
        // trace: `TELEX_KEYS` vs `BARE_KEY_ROW` — the eight Telex keys are
        // the eight letters of the Standard slot row: what one scheme types
        // tones with, the other picks with.
        let slot_letters: std::collections::BTreeSet<char> = CandidateSlotKeySet::BARE_KEY_ROW
            .iter()
            .filter_map(|key| key.chars().next())
            .filter(char::is_ascii_alphabetic)
            .collect();
        assert_eq!(
            ToneInputScheme::TELEX_KEYS
                .chars()
                .collect::<std::collections::BTreeSet<_>>(),
            slot_letters
        );
        assert!(ToneInputScheme::starts_composition('z'));
        assert!(ToneInputScheme::starts_composition('Z'));
        assert!(!ToneInputScheme::starts_composition('v'));
        assert!(!ToneInputScheme::starts_composition('f'));
    }

    #[test]
    fn raw_values_round_trip_and_are_the_mac_spellings() {
        // trace: `SettingChoice::raw` — the settings file speaks one
        // vocabulary on all three desktops (the spellings the Mac stores).
        assert_eq!(ToneInputScheme::Standard.raw(), "standard");
        assert_eq!(ToneInputScheme::Scale.raw(), "scale");
        assert_eq!(ToneInputScheme::Telex.raw(), "telex");
        for scheme in ToneInputScheme::ALL {
            assert_eq!(ToneInputScheme::from_raw(scheme.raw()), Some(*scheme));
        }
        assert_eq!(ToneInputScheme::from_raw("symbol"), Some(ToneInputScheme::Standard));
        assert_eq!(ToneInputScheme::from_raw("contour"), Some(ToneInputScheme::Scale));
        assert_eq!(ToneInputScheme::from_raw("bareKeys"), None);
    }

    #[test]
    fn scale_tone_resolves_accurately_across_lects() {
        // 四縣 / 南四縣: 24->2, 11->6, 31->3, 55->1; checked 2->3, 5->4
        assert_eq!(resolve_scale_tone(Lect::Xiien, "24", false), Some('2'));
        assert_eq!(resolve_scale_tone(Lect::Xiien, "11", false), Some('6'));
        assert_eq!(resolve_scale_tone(Lect::Xiien, "31", false), Some('3'));
        assert_eq!(resolve_scale_tone(Lect::Xiien, "55", false), Some('1'));
        assert_eq!(resolve_scale_tone(Lect::Xiien, "2", true), Some('3'));
        assert_eq!(resolve_scale_tone(Lect::Xiien, "5", true), Some('4'));

        // 海陸: 53->3, 24->2, 11->6, 33->7, 55->1; checked 2->3, 5->4
        assert_eq!(resolve_scale_tone(Lect::Hoiliug, "53", false), Some('3'));
        assert_eq!(resolve_scale_tone(Lect::Hoiliug, "24", false), Some('2'));
        assert_eq!(resolve_scale_tone(Lect::Hoiliug, "11", false), Some('6'));
        assert_eq!(resolve_scale_tone(Lect::Hoiliug, "33", false), Some('7'));
        assert_eq!(resolve_scale_tone(Lect::Hoiliug, "55", false), Some('1'));
        assert_eq!(resolve_scale_tone(Lect::Hoiliug, "2", true), Some('3'));
        assert_eq!(resolve_scale_tone(Lect::Hoiliug, "5", true), Some('4'));
    }
}
