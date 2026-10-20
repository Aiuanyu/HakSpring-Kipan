//! HagPinPlus 音節連寫个撇號（`'`）規則。
//!
//! Syllable Separator 設做「無」个時節，企盤个 HagPinPlus 音節連寫無空白，
//! 毋過會讀毋著个所在愛加撇號，像 `fad'i`、`a'i`、`n'g`。規則照客源翠
//! Romaine（`js/romanizer.js` `applyApostropheRules`），兩隻 repo 愛共樣。
//! 規格：Obs `創作/程式/HakSpring/IME/HagPinPlus`；docs/hakfa/hpp-apostrophe.md。

use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;

/// 撇號。
pub const HPP_APOSTROPHE: &str = "'";

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')
}

/// 一隻字元除忒調符、轉細寫个 ASCII 字母；毋係拉丁字母就 `None`。
fn base_letter(c: char) -> Option<char> {
    let base = c.nfd().next()?.to_ascii_lowercase();
    base.is_ascii_lowercase().then_some(base)
}

/// 前一隻音節（`prev` 个尾）摎後一隻音節（`next` 个頭）中間愛寫个分隔：
/// 會讀毋著就係 [`HPP_APOSTROPHE`]，無就係 `""`。
///
/// 前後毋係拉丁字母（漢字、標點、空白）就無加。
pub fn hpp_boundary(prev: &str, next: &str) -> &'static str {
    // 調符拆開寫（NFD）个時節，結合符號毋算字母。
    let not_mark = |c: &char| !is_combining_mark(*c);
    let Some(p) = prev.chars().rev().find(not_mark).and_then(base_letter) else {
        return "";
    };
    let mut next_letters = next.chars().filter(not_mark).map(base_letter);
    let Some(Some(n0)) = next_letters.next() else {
        return "";
    };
    let n1 = next_letters.next().flatten();
    let needs = match p {
        // 1. 子音韻尾＋母音；8. n＋g
        'b' | 'd' | 'g' | 'm' | 'n' if is_vowel(n0) => true,
        'n' if n0 == 'g' => true,
        // 2. a＋i／u
        'a' if matches!(n0, 'i' | 'u') => true,
        // 3. e＋u
        'e' if n0 == 'u' => true,
        // 4. i＋母音
        'i' if is_vowel(n0) => true,
        // 5. o＋i
        'o' if n0 == 'i' => true,
        // 6. u＋a／e／i
        'u' if matches!(n0, 'a' | 'e' | 'i') => true,
        _ => false,
    };
    // 7. 母音＋ng 開頭
    let needs = needs || (is_vowel(p) && n0 == 'n' && n1 == Some('g'));
    if needs {
        HPP_APOSTROPHE
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn join(a: &str, b: &str) -> String {
        format!("{a}{}{b}", hpp_boundary(a, b))
    }

    #[test]
    fn the_eight_romaine_rules() {
        // 1. 子音韻尾＋母音
        assert_eq!(join("fàd", "i"), "fàd'i");
        assert_eq!(join("ngiab", "ó"), "ngiab'ó");
        assert_eq!(join("hòng", "ǎ"), "hòng'ǎ");
        assert_eq!(join("sim", "u"), "sim'u");
        assert_eq!(join("san", "e"), "san'e");
        // 2. a＋i／u
        assert_eq!(join("ga", "i"), "ga'i");
        assert_eq!(join("ga", "ú"), "ga'ú");
        // 3. e＋u
        assert_eq!(join("se", "ù"), "se'ù");
        // 4. i＋母音
        assert_eq!(join("Dèd", "ǐ"), "Dèd'ǐ");
        assert_eq!(join("ki", "a"), "ki'a");
        assert_eq!(join("zii", "i"), "zii'i");
        // 5. o＋i
        assert_eq!(join("go", "í"), "go'í");
        // 6. u＋a／e／i
        assert_eq!(join("ku", "a"), "ku'a");
        assert_eq!(join("ku", "e"), "ku'e");
        assert_eq!(join("ku", "i"), "ku'i");
        // 7. 母音＋ng
        assert_eq!(join("a", "ngai"), "a'ngai");
        assert_eq!(join("hé", "ngìn"), "hé'ngìn");
        // 8. n＋g
        assert_eq!(join("san", "gin"), "san'gin");
    }

    #[test]
    fn unambiguous_joins_stay_bare() {
        assert_eq!(join("fàd", "séu"), "fàdséu");
        assert_eq!(join("he", "dui"), "hedui");
        assert_eq!(join("fǐ", "bo"), "fǐbo");
        // 規則無列个母音組合（四縣無這種韻母）
        assert_eq!(join("e", "i"), "ei");
        assert_eq!(join("o", "u"), "ou");
        assert_eq!(join("u", "o"), "uo");
        // 子音＋子音（n＋g 以外）
        assert_eq!(join("ang", "ga"), "angga");
        assert_eq!(join("ngin", "da"), "nginda");
    }

    #[test]
    fn non_latin_neighbours_add_nothing() {
        assert_eq!(hpp_boundary("這", "a"), "");
        assert_eq!(hpp_boundary("a", "係"), "");
        assert_eq!(hpp_boundary("", "a"), "");
        assert_eq!(hpp_boundary("a", ""), "");
        assert_eq!(hpp_boundary("a ", "i"), "");
        assert_eq!(hpp_boundary("a", "\u{00b7}i"), "");
    }

    #[test]
    fn decomposed_tone_marks_classify_by_their_base_letter() {
        // NFD：a + U+030C
        assert_eq!(hpp_boundary("ga\u{030c}", "i"), HPP_APOSTROPHE);
        assert_eq!(hpp_boundary("ga", "i\u{0301}"), HPP_APOSTROPHE);
    }

    #[test]
    fn whole_words_through_the_none_joiner() {
        let none = |roman| crate::api::syllable_joiner_display(roman, "");
        assert_eq!(none("fàd-séu"), "fàdséu");
        assert_eq!(none("he-dui"), "hedui");
        // 成音節 m̌ 个調符係結合符號（無預組字），後背接母音也愛隔。
        // Romaine 20261020 這兜會漏隔（正規表示式比對毋著 m + U+030C）。
        assert_eq!(none("ham-m\u{030c}-en"), "hamm\u{030c}'en");
        // 連續兩隻邊界都愛隔。Romaine 20261020 會漏後一隻（正規表示式重疊）。
        assert_eq!(none("van-sii-ǐ-i"), "vansii'ǐ'i");
        // 詞摎詞中間个空白留等，空白兩邊毋加撇號。
        assert_eq!(none("ga-i ab-a"), "ga'i ab'a");
    }
}
