//! Hakfa (客話) phonetic tables and profiles for HakSpring-Kipan.
//!
//! 定義四縣客話（含大補丸規範）之聲母、韻母、調號與合法音節切分。
//! 同時預留海陸、大埔、饒平、詔安腔調之音韻特徵。

use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

/// 客話聲母集 (Hakfa Initials)
/// - 四縣核心：b, p, m, f, v, d, t, n, l, g, k, ng, h, z, c, s
/// - HagPinPlus 係教育部客拼 (GIP) 之延伸改良，承襲其分工：
///   - j, q, x：搭配細音（高元音 i 或介音 i-）
///   - z, c, s：搭配洪音（開口呼、合口呼等非細音）
/// - 海陸/大埔/饒平/詔安擴充預留：zh, ch, sh, rh, bb
/// - 零聲母：""
pub(crate) static HAKFA_INITIALS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        // 四縣核心聲母
        "b", "p", "m", "f", "v", "d", "t", "n", "l", "g", "k", "ng", "h", "z", "c", "s",
        // 細音聲母 (GIP / HagPinPlus 正式分工)
        "j", "q", "x",
        // 他腔擴充聲母
        "zh", "ch", "sh", "rh", "bb",
        // 零聲母
        "",
    ]
    .into_iter()
    .collect()
});

/// 客話韻母集 (Hakfa Finals)
/// - 元音：a, e, i, o, u
/// - 特殊元音：ii (央元音/舌尖元音), er (小稱仔), oo, ee (詔安)
/// - 複元音：ai, au, ia, ieu, iau, ie, ien, iu, oi, ua, uai, ue, ui
/// - 陽聲韻 (鼻音韻尾 m, n, ng；詔安鼻化 nn)：
///   am, an, ang, em, en, eng, im, in, ing, on, ong, um, un, ung,
///   iam, ian, iang, iong, iun, uan, uang, uen, m, ng, nn
/// - 入聲韻：
///   - GIP 體制內與 HagPinPlus 標準：-b, -d, -g 塞音尾
///   - PFS (客語白話字) 傳統相容：-p, -t, -k 塞音尾
pub(crate) static HAKFA_FINALS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        // 單元音與特殊元音
        "a", "e", "i", "o", "u", "ii", "er", "oo", "ee",
        // 複元音 (二合與三合元音)
        "ai", "au", "eu", "ia", "ie", "io", "iu", "oi", "ua", "ue", "ui",
        "iau", "ieu", "uai",
        // 陽聲韻
        "am", "an", "ang", "em", "en", "eng", "im", "in", "ing", "on", "ong", "um", "un", "ung",
        "iam", "ian", "ien", "iang", "iong", "iun", "uan", "uang", "uen",
        // 成音節鼻音與鼻化
        "m", "ng", "ann", "enn", "inn", "onn", "unn", "iinn",
        // 入聲韻 (GIP 體制內 & HagPinPlus 標準：b, d, g 塞音尾)
        "ab", "ad", "ag", "eb", "ed", "eg", "ib", "id", "ig", "ob", "od", "og", "ub", "ud", "ug",
        "iab", "iad", "iag", "ieb", "ied", "ieg", "iog", "uab", "uad", "uag",
        // 入聲韻 (PFS 白話字傳統相容：p, t, k 塞音尾)
        "ap", "at", "ak", "ep", "et", "ek", "ip", "it", "ik", "op", "ot", "ok", "up", "ut", "uk",
        "iap", "iat", "iak", "iep", "iet", "iek", "iok", "uap", "uat", "uak",
    ]
    .into_iter()
    .collect()
});

/// 客話簡拼縮寫子音 (長度優先排序以確保多字母聲母如 zh/ch/sh/rh/ng 先行匹配)
#[allow(dead_code)]
pub(crate) static HAKFA_ABBREV_INITIALS: &[&str] = &[
    "zh", "ch", "sh", "rh", "ng", "bb",
    "b", "p", "m", "f", "v", "d", "t", "n", "l", "g", "k", "h", "z", "c", "s", "j", "q", "x",
];

/// 客話四縣調號與變音符號 (HagPinPlus)
/// - 1: 陰平 24 -> 銳音 (acute \u{0301})
/// - 2: 上聲 31 -> 重音 (grave \u{0300})
/// - 3: 去聲 55 -> 無符號
/// - 4: 陰入 2  -> 重音 (grave \u{0300})
/// - 5: 陽平 11 -> 抑揚符 (caron \u{030c})
/// - 8: 陽入 5  -> 無符號
#[allow(dead_code)]
pub(crate) static HAKFA_TONE_NUM_TO_COMBINING: Lazy<HashMap<&'static str, &'static str>> =
    Lazy::new(|| {
        [
            ("1", "\u{0301}"),
            ("2", "\u{0300}"),
            ("3", ""),
            ("4", "\u{0300}"),
            ("5", "\u{030c}"),
            ("8", ""),
            // 亦相容台羅 1~9
            ("6", "\u{030c}"),
            ("7", "\u{0304}"),
            ("9", "\u{0306}"),
        ]
        .into_iter()
        .collect()
    });

/// 結合符號轉客話調號
pub(crate) static HAKFA_COMBINING_TO_TONE_NUM: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    [
        ('\u{0301}', "1"), // 銳音 -> 陰平 24
        ('\u{0300}', "2"), // 重音 -> 上聲 31 (或陰入 4)
        ('\u{030c}', "5"), // 抑揚 -> 陽平 11
        ('\u{0304}', "7"), // 長音 -> 陽去
        ('\u{0306}', "9"), // 短音
    ]
    .into_iter()
    .collect()
});

/// 切分客話音節為 (聲母, 韻母)
pub(crate) fn split_hakfa_initial_final(text: &str) -> Option<(&str, &str)> {
    for i in 0..=text.len() {
        if !text.is_char_boundary(i) {
            continue;
        }
        let initial = &text[..i];
        if HAKFA_INITIALS.contains(initial) {
            let final_str = &text[i..];
            if HAKFA_FINALS.contains(final_str) {
                return Some((initial, final_str));
            }
        }
    }
    None
}

/// 驗證是否為合法客話音節
pub fn is_valid_hakfa_syllable(syllable: &str) -> bool {
    let clean = syllable.trim().to_lowercase();
    split_hakfa_initial_final(&clean).is_some()
}

use unicode_normalization::UnicodeNormalization;

/// 客話專屬音節調號提取：依 HagPinPlus 聲調表提取
pub fn strip_hakfa_tone_mark(text: &str) -> (String, String) {
    let mut bare = String::new();
    let mut tone_num = String::new();
    let nfd: String = text.nfd().collect();
    for ch in nfd.chars() {
        if let Some(&num) = HAKFA_COMBINING_TO_TONE_NUM.get(&ch) {
            tone_num = num.to_string();
        } else if !crate::normalization::is_combining_tone_mark(ch) {
            bare.push(ch);
        }
    }
    let nfc_bare: String = bare.nfc().collect();
    // 檢查結尾是否有 ASCII 數字調 (1..=8)
    if tone_num.is_empty() {
        if let Some(last_ch) = nfc_bare.chars().last() {
            if last_ch.is_ascii_digit() {
                let val = last_ch.to_string();
                let without_digit = nfc_bare[..nfc_bare.len() - 1].to_string();
                return (without_digit, val);
            }
        }
    }
    // 入聲判斷：若為重音符號且結尾是塞音 (b, d, g, p, t, k)，四縣為陰入 (4)
    if tone_num == "2" {
        if let Some((_, fin)) = split_hakfa_initial_final(&nfc_bare) {
            if fin.ends_with('b') || fin.ends_with('d') || fin.ends_with('g')
                || fin.ends_with('p') || fin.ends_with('t') || fin.ends_with('k') {
                tone_num = "4".to_string();
            }
        }
    }
    (nfc_bare, tone_num)
}

/// 客話音節正規化：提取調號與無調客話音節
/// 回傳 (canonical_toneless, tone_digit)
pub fn canonicalize_hakfa_syllable(token: &str) -> Option<(String, String)> {
    let (bare, tone) = strip_hakfa_tone_mark(token);
    let lowered = bare.to_lowercase();
    if is_valid_hakfa_syllable(&lowered) {
        Some((lowered, tone))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hakfa_initials_and_finals() {
        assert!(HAKFA_INITIALS.contains("f"));
        assert!(HAKFA_INITIALS.contains("v"));
        assert!(HAKFA_INITIALS.contains("z"));
        assert!(HAKFA_INITIALS.contains("c"));
        assert!(HAKFA_INITIALS.contains("ng"));
        assert!(HAKFA_INITIALS.contains("j"));
        assert!(HAKFA_INITIALS.contains("q"));
        assert!(HAKFA_INITIALS.contains("x"));
        assert!(HAKFA_INITIALS.contains("zh"));
        assert!(HAKFA_INITIALS.contains("ch"));
        assert!(HAKFA_INITIALS.contains("sh"));
        assert!(HAKFA_INITIALS.contains("rh"));
        assert!(HAKFA_INITIALS.contains("bb"));
        assert!(HAKFA_INITIALS.contains(""));

        assert!(HAKFA_FINALS.contains("ii"));
        assert!(HAKFA_FINALS.contains("ad"));
        assert!(HAKFA_FINALS.contains("ag"));
        assert!(HAKFA_FINALS.contains("ab"));
        assert!(HAKFA_FINALS.contains("er"));
        assert!(HAKFA_FINALS.contains("ien"));
        assert!(HAKFA_FINALS.contains("ian"));
        assert!(HAKFA_FINALS.contains("eu"));
        assert!(HAKFA_FINALS.contains("iog"));
    }

    #[test]
    fn test_split_hakfa_syllables() {
        // 發燒: fad, seu
        assert_eq!(split_hakfa_initial_final("fad"), Some(("f", "ad")));
        assert_eq!(split_hakfa_initial_final("seu"), Some(("s", "eu")));
        // 醫院: i, ien / i, ian
        assert_eq!(split_hakfa_initial_final("i"), Some(("", "i")));
        assert_eq!(split_hakfa_initial_final("ien"), Some(("", "ien")));
        assert_eq!(split_hakfa_initial_final("ian"), Some(("", "ian")));
        // 肚屎: du, sii
        assert_eq!(split_hakfa_initial_final("du"), Some(("d", "u")));
        assert_eq!(split_hakfa_initial_final("sii"), Some(("s", "ii")));
        // 恁仔細: an, zii, se
        assert_eq!(split_hakfa_initial_final("an"), Some(("", "an")));
        assert_eq!(split_hakfa_initial_final("zii"), Some(("z", "ii")));
        assert_eq!(split_hakfa_initial_final("se"), Some(("s", "e")));
        // 額頭: ngiag, teu
        assert_eq!(split_hakfa_initial_final("ngiag"), Some(("ng", "iag")));
        assert_eq!(split_hakfa_initial_final("teu"), Some(("t", "eu")));
        // 跛腳仔: bai, giog, e
        assert_eq!(split_hakfa_initial_final("bai"), Some(("b", "ai")));
        assert_eq!(split_hakfa_initial_final("giog"), Some(("g", "iog")));
        assert_eq!(split_hakfa_initial_final("e"), Some(("", "e")));

        // 體制內客拼/老派相容 (j, q, x)
        assert_eq!(split_hakfa_initial_final("ji"), Some(("j", "i")));
        assert_eq!(split_hakfa_initial_final("qia"), Some(("q", "ia")));
        assert_eq!(split_hakfa_initial_final("xien"), Some(("x", "ien")));

        // 他腔 (海陸/詔安)
        assert_eq!(split_hakfa_initial_final("zha"), Some(("zh", "a")));
        assert_eq!(split_hakfa_initial_final("shu"), Some(("sh", "u")));
        assert_eq!(split_hakfa_initial_final("rhi"), Some(("rh", "i")));
        assert_eq!(split_hakfa_initial_final("bbu"), Some(("bb", "u")));
    }

    #[test]
    fn test_canonicalize_hakfa_syllable() {
        assert_eq!(
            canonicalize_hakfa_syllable("fàd"),
            Some(("fad".to_string(), "4".to_string()))
        );
        assert_eq!(
            canonicalize_hakfa_syllable("sìi"),
            Some(("sii".to_string(), "2".to_string()))
        );
        assert_eq!(
            canonicalize_hakfa_syllable("séu"),
            Some(("seu".to_string(), "1".to_string()))
        );
        assert_eq!(
            canonicalize_hakfa_syllable("hǎ"),
            Some(("ha".to_string(), "5".to_string()))
        );
        assert_eq!(
            canonicalize_hakfa_syllable("i"),
            Some(("i".to_string(), "".to_string()))
        );
        assert_eq!(
            canonicalize_hakfa_syllable("ien"),
            Some(("ien".to_string(), "".to_string()))
        );
        assert!(canonicalize_hakfa_syllable("xyz123").is_none());
    }
}
