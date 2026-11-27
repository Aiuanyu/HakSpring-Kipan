#![forbid(unsafe_code)]
//! Unified dual-script two-line converter for HakSpring-Kipan and HakSpring Web.
//!
//! Converts inline dual-script annotations (e.g. `字詞 (拼音)`, `拼音 (字詞)`,
//! `[...]`, `{...}`, `（...）`, `<ruby>字<rt>im</rt></ruby>`) into clean
//! parallel two-line output (Hanji line + Romanization line), while preserving
//! unannotated English/alphanumeric tokens on both lines and maintaining
//! bidirectional full-width / half-width punctuation alignment.

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pair {
    pub han: String,
    pub rom: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedLine {
    pub order: String,
    pub han_line: String,
    pub rom_line: String,
    pub top: String,
    pub bottom: String,
    pub pairs: Vec<Pair>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversionResult {
    pub top_text: String,
    pub bottom_text: String,
    pub combined_text: String,
    pub top_lines: Vec<String>,
    pub bottom_lines: Vec<String>,
    pub pairs: Vec<Pair>,
    pub detected_order: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetSlice {
    /// Length in UTF-16 code units (for Android `InputConnection.deleteSurroundingText`).
    pub delete_before_utf16_len: usize,
    pub text: String,
}

static HAN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\p{Script=Han}").unwrap());
static LATIN_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[a-zA-Z\u{00C0}-\u{024F}\u{1E00}-\u{1EFF}]").unwrap());
static BRACKET_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\(（\[［\{｛]([^\(\)（）\[\]［］\{\}｛\}\r\n]+?)[\)）\]］\}｝]").unwrap()
});
static HAN_NEUTRAL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^([\s\S]*?[^\p{Script=Han}\s])\s*([\p{Script=Han}]+)$").unwrap());
static ROM_PUNCT_SPACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[,.!?;:]").unwrap());

static FULL_RUBY_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?i)<ruby[^>]*>(?:<rb>)?([^<]+?)(?:</rb>)?(?:\s*<rp>[^<]*</rp>\s*)?<rt[^>]*>([^<]+?)</rt>(?:\s*<rp>[^<]*</rp>\s*)?</ruby>",
    )
    .unwrap()
});
static SHORT_RUBY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)([^<>\s]+)<rt[^>]*>([^<]+?)</rt>").unwrap());
static LEFTOVER_RUBY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)</?(?:ruby|rb|rt|rp)[^>]*>").unwrap());

const PUNCT_CHARS: &str = "，。、；：？！…⋯‥—–―～〜·‧・／＼＋＝＜＞＆％＃＠＊＿｜｀＄＾＇＂－「」『』“”‘’《》〈〉（）〔〕【】［］｛｝〖〗〘〙〚〛〝〞﹁﹂﹃﹄﹒﹑﹐﹗﹖﹔﹕︰﹙﹚﹛﹜．｡､,.?!;:\"'()[]{}~/\\+&%#@*_|$^";

const OPEN_PUNCT: &[char] = &[
    '「', '『', '“', '‘', '《', '〈', '（', '〔', '【', '［', '｛', '〖', '〘', '〚', '〝', '﹁',
    '﹃', '﹙', '﹛', '(', '[', '{',
];

const CLOSE_PUNCT: &[char] = &[
    '」', '』', '”', '’', '》', '〉', '）', '〕', '】', '］', '｝', '〗', '〙', '〛', '〞', '﹂',
    '﹄', '﹚', '﹜', ')', ']', '}',
];

static HAKFA_INITIALS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "b", "p", "m", "f", "v", "d", "t", "n", "l", "g", "k", "ng", "h", "z", "c", "s", "j", "q",
        "x", "zh", "ch", "sh", "rh", "bb", "",
    ]
    .into_iter()
    .collect()
});

static HAKFA_FINALS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "a", "e", "i", "o", "u", "ii", "er", "oo", "ee", "ai", "au", "eu", "ia", "ie", "io", "iu",
        "oi", "ua", "ue", "ui", "iau", "ieu", "uai", "am", "an", "ang", "em", "en", "eng", "im",
        "in", "ing", "on", "ong", "um", "un", "ung", "iam", "ian", "ien", "iang", "iong", "iun",
        "uan", "uang", "uen", "iim", "iin", "iing", "m", "ng", "ann", "enn", "inn", "onn", "unn",
        "iinn", "ab", "ad", "ag", "eb", "ed", "eg", "ib", "id", "ig", "ob", "od", "og", "ub", "ud",
        "ug", "iab", "iad", "iag", "ieb", "ied", "ieg", "iog", "uab", "uad", "uag", "iib", "iid",
        "iig", "ap", "at", "ak", "ep", "et", "ek", "ip", "it", "ik", "op", "ot", "ok", "up", "ut",
        "uk", "iap", "iat", "iak", "iep", "iet", "iek", "iok", "uap", "uat", "uak", "iip", "iit",
        "iik",
    ]
    .into_iter()
    .collect()
});

fn is_punct_char(c: char) -> bool {
    PUNCT_CHARS.contains(c)
}

fn is_punct_or_space(c: char) -> bool {
    c.is_whitespace() || is_punct_char(c)
}

fn is_open_punct_char(c: char) -> bool {
    OPEN_PUNCT.contains(&c)
}

fn is_close_punct_char(c: char) -> bool {
    CLOSE_PUNCT.contains(&c)
}

fn ends_with_space_or_open_punct(s: &str) -> bool {
    match s.chars().last() {
        None => true,
        Some(c) => c.is_whitespace() || is_open_punct_char(c),
    }
}

fn ends_with_cjk_punct(s: &str) -> bool {
    match s.chars().last() {
        None => false,
        Some(c) => {
            matches!(
                c,
                '，' | '。'
                    | '、'
                    | '；'
                    | '：'
                    | '？'
                    | '！'
                    | '…'
                    | '—'
                    | '～'
                    | '·'
                    | '「'
                    | '」'
                    | '『'
                    | '』'
                    | '《'
                    | '》'
                    | '〈'
                    | '〉'
                    | '（'
                    | '）'
                    | '【'
                    | '】'
                    | '［'
                    | '］'
                    | '｛'
                    | '｝'
            )
        }
    }
}

pub fn has_han(s: &str) -> bool {
    HAN_REGEX.is_match(s)
}

pub fn has_latin(s: &str) -> bool {
    LATIN_REGEX.is_match(s)
}

fn count_han(s: &str) -> usize {
    HAN_REGEX.find_iter(s).count()
}

/// Converts CJK full-width punctuation to half-width romance punctuation.
pub fn convert_punct_to_romance(p: &str) -> String {
    if p.is_empty() {
        return String::new();
    }
    const REPLACEMENTS: &[(&str, &str)] = &[
        ("……", "..."),
        ("…", "..."),
        ("⋯⋯", "..."),
        ("⋯", "..."),
        ("‥", ".."),
        ("——", "--"),
        ("—", "-"),
        ("–", "-"),
        ("―", "-"),
        ("－", "-"),
        ("～", "~"),
        ("〜", "~"),
        ("·", "."),
        ("‧", "."),
        ("・", "."),
        ("、", ","),
        ("﹑", ","),
        ("､", ","),
        ("，", ","),
        ("﹐", ","),
        ("。", "."),
        ("．", "."),
        ("﹒", "."),
        ("｡", "."),
        ("！", "!"),
        ("﹗", "!"),
        ("？", "?"),
        ("﹖", "?"),
        ("；", ";"),
        ("﹔", ";"),
        ("：", ":"),
        ("﹕", ":"),
        ("︰", ":"),
        ("（", "("),
        ("﹙", "("),
        ("）", ")"),
        ("﹚", ")"),
        ("【", "["),
        ("〔", "["),
        ("［", "["),
        ("〖", "["),
        ("〘", "["),
        ("〚", "["),
        ("】", "]"),
        ("〕", "]"),
        ("］", "]"),
        ("〗", "]"),
        ("〙", "]"),
        ("〛", "]"),
        ("｛", "{"),
        ("﹛", "{"),
        ("｝", "}"),
        ("﹜", "}"),
        ("「", "\""),
        ("“", "\""),
        ("《", "\""),
        ("〝", "\""),
        ("＂", "\""),
        ("﹁", "\""),
        ("」", "\""),
        ("”", "\""),
        ("》", "\""),
        ("〞", "\""),
        ("﹂", "\""),
        ("『", "'"),
        ("‘", "'"),
        ("〈", "'"),
        ("＇", "'"),
        ("﹃", "'"),
        ("』", "'"),
        ("’", "'"),
        ("〉", "'"),
        ("﹄", "'"),
        ("／", "/"),
        ("＼", "\\"),
        ("＋", "+"),
        ("＝", "="),
        ("＜", "<"),
        ("＞", ">"),
        ("＆", "&"),
        ("％", "%"),
        ("＃", "#"),
        ("＠", "@"),
        ("＊", "*"),
        ("＿", "_"),
        ("｜", "|"),
        ("｀", "`"),
        ("＄", "$"),
        ("＾", "^"),
    ];
    let mut result = p.to_string();
    for (target, rep) in REPLACEMENTS {
        if result.contains(target) {
            result = result.replace(target, rep);
        }
    }
    result
}

/// Converts half-width romance punctuation to CJK full-width punctuation.
pub fn convert_punct_to_han(p: &str) -> String {
    if p.is_empty() {
        return String::new();
    }
    const REPLACEMENTS: &[(&str, &str)] = &[
        ("...", "……"),
        ("--", "——"),
        ("~", "～"),
        (",", "，"),
        (".", "。"),
        ("!", "！"),
        ("?", "？"),
        (";", "；"),
        (":", "："),
        ("(", "（"),
        (")", "）"),
        ("[", "［"),
        ("]", "］"),
        ("{", "｛"),
        ("}", "｝"),
        ("/", "／"),
        ("\\", "＼"),
        ("+", "＋"),
        ("=", "＝"),
        ("&", "＆"),
        ("%", "％"),
        ("#", "＃"),
        ("@", "＠"),
        ("*", "＊"),
        ("_", "＿"),
        ("|", "｜"),
        ("$", "＄"),
        ("^", "＾"),
    ];
    let mut result = p.to_string();
    for (target, rep) in REPLACEMENTS {
        if result.contains(target) {
            result = result.replace(target, rep);
        }
    }
    result
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SplitBetweenResult {
    prev_punct: String,
    curr_open: String,
    core: String,
    curr_close: String,
    trailing_space: String,
}

fn split_between(between: &str) -> SplitBetweenResult {
    if between.is_empty() {
        return SplitBetweenResult {
            prev_punct: String::new(),
            curr_open: String::new(),
            core: String::new(),
            curr_close: String::new(),
            trailing_space: String::new(),
        };
    }

    let chars: Vec<char> = between.chars().collect();
    let mut start = 0;
    while start < chars.len() && is_punct_or_space(chars[start]) {
        start += 1;
    }

    let mut end = chars.len();
    while end > start && is_punct_or_space(chars[end - 1]) {
        end -= 1;
    }

    let leading = &chars[..start];
    let core: String = chars[start..end].iter().collect();
    let trailing = &chars[end..];

    let mut open_idx = leading.len();
    while open_idx > 0 && is_open_punct_char(leading[open_idx - 1]) {
        open_idx -= 1;
    }
    let prev_punct: String = leading[..open_idx].iter().collect();
    let curr_open: String = leading[open_idx..].iter().collect();

    let mut close_idx = 0;
    while close_idx < trailing.len() && is_close_punct_char(trailing[close_idx]) {
        close_idx += 1;
    }
    let curr_close: String = trailing[..close_idx].iter().collect();
    let trailing_space: String = trailing[close_idx..].iter().collect();

    SplitBetweenResult {
        prev_punct,
        curr_open,
        core,
        curr_close,
        trailing_space,
    }
}

/// Normalizes HTML `<ruby>` and `<rt>` tags into `word (reading)` bracket format.
pub fn normalize_ruby(text: &str) -> String {
    let lower = text.to_ascii_lowercase();
    if !lower.contains("<rt") && !lower.contains("<ruby") {
        return text.to_string();
    }
    let step1 = FULL_RUBY_REGEX.replace_all(text, "$1 ($2)");
    let step2 = SHORT_RUBY_REGEX.replace_all(&step1, "$1 ($2)");
    let step3 = LEFTOVER_RUBY_REGEX.replace_all(&step2, "");
    step3.into_owned()
}

struct TokenItem {
    between: String,
    inside: String,
}

fn tokenize_line(line: &str) -> (Vec<TokenItem>, String) {
    let mut tokens = Vec::new();
    let mut last_end = 0;
    for cap in BRACKET_REGEX.captures_iter(line) {
        let full = cap.get(0).unwrap();
        let inside = cap.get(1).unwrap().as_str().to_string();
        let between = line[last_end..full.start()].to_string();
        tokens.push(TokenItem { between, inside });
        last_end = full.end();
    }
    let trailing = line[last_end..].to_string();
    (tokens, trailing)
}

/// Scores tokens to determine whether Hanji is outside brackets (`true`) or inside (`false`).
/// Returns `Some(true/false)` when unambiguous, or `None` if all tokens are script-neutral.
fn score_is_han_outside(tokens: &[TokenItem]) -> Option<bool> {
    let mut han_outside_votes: i32 = 0;
    let mut rom_outside_votes: i32 = 0;

    for tok in tokens {
        let between_han = has_han(&tok.between);
        let between_latin = has_latin(&tok.between);
        let inside_han = has_han(&tok.inside);
        let inside_latin = has_latin(&tok.inside);

        if inside_han && !inside_latin && between_latin {
            rom_outside_votes += 2;
        } else if between_han && !inside_han && inside_latin {
            han_outside_votes += 2;
        } else if between_han && !inside_han {
            han_outside_votes += 1;
        } else if between_latin && inside_han {
            rom_outside_votes += 1;
        }
    }

    if han_outside_votes > rom_outside_votes {
        Some(true)
    } else if rom_outside_votes > han_outside_votes {
        Some(false)
    } else {
        None
    }
}

fn is_valid_hakfa_syllable_ascii(lower_ascii: &str) -> bool {
    for i in 0..=lower_ascii.len() {
        if !lower_ascii.is_char_boundary(i) {
            continue;
        }
        let initial = &lower_ascii[..i];
        if HAKFA_INITIALS.contains(initial) {
            let final_str = &lower_ascii[i..];
            if HAKFA_FINALS.contains(final_str) {
                return true;
            }
        }
    }
    false
}

fn can_segment_hakfa_ascii(s: &str) -> bool {
    if s.is_empty() || !s.is_ascii() {
        return false;
    }
    let n = s.len();
    let mut dp = vec![false; n + 1];
    dp[0] = true;
    for i in 1..=n {
        for j in 0..i {
            if dp[j] && is_valid_hakfa_syllable_ascii(&s[j..i]) {
                dp[i] = true;
                break;
            }
        }
    }
    dp[n]
}

fn strip_tone_to_ascii(s: &str) -> (String, bool) {
    let mut out = String::with_capacity(s.len());
    let mut has_tone = false;
    for c in s.nfd() {
        if ('\u{0300}'..='\u{036F}').contains(&c) {
            has_tone = true;
        } else {
            out.push(c);
        }
    }
    (out, has_tone)
}

fn has_internal_uppercase(word: &str) -> bool {
    let mut chars = word.chars();
    let Some(_first) = chars.next() else {
        return false;
    };
    chars.any(|c| c.is_ascii_uppercase())
}

fn is_all_uppercase_ascii(word: &str) -> bool {
    let letters: Vec<char> = word.chars().filter(|c| c.is_ascii_alphabetic()).collect();
    letters.len() >= 2 && letters.iter().all(|c| c.is_ascii_uppercase())
}

fn count_approx_hakfa_syllables(token: &str) -> usize {
    let clean = token.trim_matches(|c: char| is_punct_char(c) || c.is_whitespace());
    if clean.is_empty() {
        return 0;
    }
    let parts: Vec<&str> = clean
        .split(['-', '\'', '’'])
        .filter(|p| !p.is_empty())
        .collect();
    let mut count = 0;
    for part in parts {
        let (ascii_part, _) = strip_tone_to_ascii(part);
        let lower = ascii_part.to_ascii_lowercase();
        if is_valid_hakfa_syllable_ascii(&lower) {
            count += 1;
        } else {
            // Connected multi-syllable word (e.g. `cǐifi`, `dídù`): estimate by vowel groups.
            let mut in_vowel = false;
            let mut groups = 0;
            for ch in lower.chars() {
                let is_v = matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u');
                if is_v && !in_vowel {
                    groups += 1;
                }
                in_vowel = is_v;
            }
            count += groups.max(1);
        }
    }
    count
}

fn is_definitely_neutral_rom_token(token: &str) -> bool {
    let trimmed = token.trim_matches(|c: char| is_punct_char(c) || c.is_whitespace());
    if trimmed.is_empty() {
        return false;
    }
    if has_han(trimmed) {
        return true;
    }
    if trimmed.chars().any(|c| {
        c.is_ascii_digit()
            || matches!(
                c,
                '.' | '/' | '+' | '=' | '@' | '#' | '$' | '%' | '&' | '*' | '_' | '\\' | '|'
            )
    }) {
        return true;
    }

    let (ascii_word, has_tone) = strip_tone_to_ascii(trimmed);
    if has_tone {
        // Tone diacritics indicate Hakfa romanization.
        return false;
    }

    if has_internal_uppercase(&ascii_word) || is_all_uppercase_ascii(&ascii_word) {
        return true;
    }

    let parts: Vec<&str> = ascii_word
        .split(['-', '\'', '’'])
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        return true;
    }
    for part in parts {
        let lower = part.to_ascii_lowercase();
        if !can_segment_hakfa_ascii(&lower) {
            return true;
        }
    }
    false
}

/// Splits `core` into `(neutral_prefix, effective_core)` when `is_han_outside == false`.
fn split_rom_neutral_and_core(core: &str, inside_han: &str) -> (String, String) {
    let han_count = count_han(inside_han);

    // If `core` contains a leading Hanji/punctuation prefix without whitespace before Latin
    // (e.g. `試驗：Ngǎi`), peel off everything up to the start of the Latin segment.
    let mut prefix_offset = 0;
    if has_han(core) {
        if let Some(m) = LATIN_REGEX.find(core) {
            if has_han(&core[..m.start()]) {
                prefix_offset = m.start();
            }
        }
    }
    let prefix_from_han = core[..prefix_offset].trim();
    let rest = &core[prefix_offset..];

    // Collect whitespace-separated token byte ranges (start, end) in `rest`.
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut in_tok = false;
    let mut tok_start = 0;
    for (idx, ch) in rest.char_indices() {
        if ch.is_whitespace() {
            if in_tok {
                spans.push((tok_start, idx));
                in_tok = false;
            }
        } else if !in_tok {
            tok_start = idx;
            in_tok = true;
        }
    }
    if in_tok {
        spans.push((tok_start, rest.len()));
    }

    if spans.len() <= 1 {
        return (prefix_from_han.to_string(), rest.to_string());
    }

    let mut split_idx = 0;
    for i in 0..(spans.len() - 1) {
        let tok = &rest[spans[i].0..spans[i].1];
        if is_definitely_neutral_rom_token(tok) {
            split_idx = i + 1;
            continue;
        }

        // Handle coincidental ASCII English words (e.g. `Go`, `Note`, `Code`) when
        // the remaining tokens already cover `han_count` syllables and `tok` has no tone mark.
        let (_, has_tone) = strip_tone_to_ascii(tok);
        if !has_tone && han_count > 0 {
            let remaining_syllables: usize = spans[(i + 1)..]
                .iter()
                .map(|&(s, e)| count_approx_hakfa_syllables(&rest[s..e]))
                .sum();
            if remaining_syllables >= han_count {
                split_idx = i + 1;
                continue;
            }
        }

        break;
    }

    if split_idx == 0 {
        (prefix_from_han.to_string(), rest.to_string())
    } else {
        let neutral_end = spans[split_idx - 1].1;
        let core_start = spans[split_idx].0;
        let combined_neutral = if prefix_from_han.is_empty() {
            rest[..neutral_end].trim().to_string()
        } else {
            format!("{} {}", prefix_from_han, rest[..neutral_end].trim())
        };
        (combined_neutral, rest[core_start..].to_string())
    }
}

fn is_internal_token_punct(chars: &[char], idx: usize) -> bool {
    let c = chars[idx];
    if !matches!(c, '.' | '-' | '_' | '/' | '+' | ':') {
        return false;
    }
    let prev_alnum = idx > 0 && chars[idx - 1].is_ascii_alphanumeric();
    let next_alnum = idx + 1 < chars.len() && chars[idx + 1].is_ascii_alphanumeric();
    prev_alnum && next_alnum
}

/// Emits a neutral/unannotated segment (which may contain English words, numbers, spaces,
/// and punctuation) into both `han_line` and `rom_line`.
fn emit_neutral_segment(seg: &str, han_line: &mut String, rom_line: &mut String) {
    let trimmed = seg.trim();
    if trimmed.is_empty() {
        return;
    }

    let chars: Vec<char> = trimmed.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        if is_punct_char(c) && !is_internal_token_punct(&chars, i) {
            let start = i;
            while i < chars.len() && is_punct_char(chars[i]) && !is_internal_token_punct(&chars, i)
            {
                i += 1;
            }
            let p: String = chars[start..i].iter().collect();
            han_line.push_str(&convert_punct_to_han(&p));
            let rom_p = convert_punct_to_romance(&p);
            rom_line.push_str(&rom_p);
            let has_more_non_close = chars[i..]
                .iter()
                .any(|&ch| !ch.is_whitespace() && !is_close_punct_char(ch));
            if has_more_non_close
                && !rom_line.ends_with(' ')
                && ROM_PUNCT_SPACE_REGEX.is_match(&rom_p)
            {
                rom_line.push(' ');
            }
        } else {
            let start = i;
            while i < chars.len()
                && !chars[i].is_whitespace()
                && (!is_punct_char(chars[i]) || is_internal_token_punct(&chars, i))
            {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            if !ends_with_space_or_open_punct(han_line) && !ends_with_cjk_punct(han_line) {
                han_line.push(' ');
            }
            han_line.push_str(&word);

            if !ends_with_space_or_open_punct(rom_line) {
                rom_line.push(' ');
            }
            rom_line.push_str(&word);
        }
    }
}

/// Parses a single line of inline dual-script Kipan text.
pub fn parse_kipan_line(
    raw_line: &str,
    auto_space_romance: bool,
    force_order: &str,
) -> ParsedLine {
    parse_kipan_line_with_default(raw_line, auto_space_romance, force_order, true)
}

fn parse_kipan_line_with_default(
    raw_line: &str,
    _auto_space_romance: bool,
    force_order: &str,
    default_han_outside: bool,
) -> ParsedLine {
    let line = normalize_ruby(raw_line);
    let (tokens, trailing) = tokenize_line(&line);

    if tokens.is_empty() {
        let final_order = if force_order == "auto" {
            if default_han_outside {
                "han-top".to_string()
            } else {
                "rom-top".to_string()
            }
        } else {
            force_order.to_string()
        };
        return ParsedLine {
            order: final_order,
            han_line: line.clone(),
            rom_line: line.clone(),
            top: line.clone(),
            bottom: line,
            pairs: Vec::new(),
        };
    }

    let is_han_outside = score_is_han_outside(&tokens).unwrap_or(default_han_outside);

    let mut han_line = String::new();
    let mut rom_line = String::new();
    let mut pairs = Vec::with_capacity(tokens.len());

    for (i, tok) in tokens.iter().enumerate() {
        let split = split_between(&tok.between);
        let inside = &tok.inside;

        // 1. Previous punctuation (prev_punct) and inter-word spacing
        if !split.prev_punct.is_empty() {
            let p: String = split
                .prev_punct
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();
            if !p.is_empty() {
                han_line.push_str(&convert_punct_to_han(&p));
                let rom_p = convert_punct_to_romance(&p);
                rom_line.push_str(&rom_p);
                if !rom_line.ends_with(' ') && ROM_PUNCT_SPACE_REGEX.is_match(&rom_p) {
                    rom_line.push(' ');
                }
            } else if i > 0 && !rom_line.ends_with(' ') {
                rom_line.push(' ');
            }
        } else if i > 0 && !rom_line.ends_with(' ') {
            rom_line.push(' ');
        }

        // 2. Opening punctuation/brackets for this token (curr_open)
        if !split.curr_open.is_empty() {
            han_line.push_str(&convert_punct_to_han(&split.curr_open));
            rom_line.push_str(&convert_punct_to_romance(&split.curr_open));
        }

        // 3. Separate unannotated neutral prefix (English/alphanumeric/version) from annotated core
        let (neutral, effective_core) = if is_han_outside {
            if let Some(caps) = HAN_NEUTRAL_REGEX.captures(&split.core) {
                let neutral_part = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
                let core_part = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                if !neutral_part.is_empty() {
                    (neutral_part.to_string(), core_part.to_string())
                } else {
                    (String::new(), split.core.clone())
                }
            } else {
                (String::new(), split.core.clone())
            }
        } else {
            split_rom_neutral_and_core(&split.core, inside)
        };

        if !neutral.is_empty() {
            emit_neutral_segment(&neutral, &mut han_line, &mut rom_line);
        }

        let (han_word, rom_word) = if is_han_outside {
            (effective_core, inside.clone())
        } else {
            (inside.clone(), effective_core)
        };

        // Ensure clean spacing before `han_word` when preceded by neutral ASCII text or ASCII `han_word`.
        if !han_word.is_empty()
            && !ends_with_space_or_open_punct(&han_line)
            && !ends_with_cjk_punct(&han_line)
            && (!neutral.is_empty() || (!has_han(&han_word) && has_latin(&han_word)))
        {
            han_line.push(' ');
        }
        if !rom_word.is_empty() && !ends_with_space_or_open_punct(&rom_line) && !neutral.is_empty()
        {
            rom_line.push(' ');
        }

        han_line.push_str(&han_word);
        rom_line.push_str(&rom_word);

        // 4. Closing punctuation/brackets for this token (curr_close)
        if !split.curr_close.is_empty() {
            han_line.push_str(&convert_punct_to_han(&split.curr_close));
            rom_line.push_str(&convert_punct_to_romance(&split.curr_close));
        }

        pairs.push(Pair {
            han: han_word,
            rom: rom_word,
        });
    }

    // 5. Trailing punctuation and unannotated words (e.g. sentence-final English words / version strings)
    if !trailing.is_empty() {
        emit_neutral_segment(&trailing, &mut han_line, &mut rom_line);
    }

    let final_order = if force_order == "auto" {
        if is_han_outside {
            "han-top".to_string()
        } else {
            "rom-top".to_string()
        }
    } else {
        force_order.to_string()
    };

    let han_trimmed = han_line.trim().to_string();
    let rom_trimmed = rom_line.trim().to_string();
    let (top, bottom) = if final_order == "han-top" {
        (han_trimmed.clone(), rom_trimmed.clone())
    } else {
        (rom_trimmed.clone(), han_trimmed.clone())
    };

    ParsedLine {
        order: final_order,
        han_line: han_trimmed,
        rom_line: rom_trimmed,
        top,
        bottom,
        pairs,
    }
}

/// Converts multi-line Kipan inline-annotated text into parallel two-line output.
pub fn convert_kipan_text(
    text: &str,
    auto_space_romance: bool,
    force_order: &str,
) -> ConversionResult {
    if text.is_empty() {
        return ConversionResult {
            top_text: String::new(),
            bottom_text: String::new(),
            combined_text: String::new(),
            top_lines: Vec::new(),
            bottom_lines: Vec::new(),
            pairs: Vec::new(),
            detected_order: "han-top".to_string(),
        };
    }

    let lines: Vec<&str> = text.split('\n').map(|l| l.strip_suffix('\r').unwrap_or(l)).collect();

    // Determine document-level `default_han_outside` from the first line with an unambiguous script order.
    let mut doc_han_outside = true;
    for line in &lines {
        if line.trim().is_empty() {
            continue;
        }
        let normalized = normalize_ruby(line);
        let (tokens, _) = tokenize_line(&normalized);
        if let Some(han_out) = score_is_han_outside(&tokens) {
            doc_han_outside = han_out;
            break;
        }
    }

    let mut top_lines = Vec::with_capacity(lines.len());
    let mut bottom_lines = Vec::with_capacity(lines.len());
    let mut detected_order = if force_order == "auto" {
        if doc_han_outside {
            "han-top".to_string()
        } else {
            "rom-top".to_string()
        }
    } else {
        force_order.to_string()
    };
    let mut all_pairs = Vec::new();
    let mut first_non_empty_seen = false;

    for line in lines {
        if line.trim().is_empty() {
            top_lines.push(String::new());
            bottom_lines.push(String::new());
            continue;
        }

        let res = parse_kipan_line_with_default(line, auto_space_romance, force_order, doc_han_outside);
        if !first_non_empty_seen {
            detected_order = res.order.clone();
            first_non_empty_seen = true;
        }
        top_lines.push(res.top);
        bottom_lines.push(res.bottom);
        all_pairs.extend(res.pairs);
    }

    let top_text = top_lines.join("\n");
    let bottom_text = bottom_lines.join("\n");
    let combined_text = format!("{top_text}\n{bottom_text}");

    ConversionResult {
        top_text,
        bottom_text,
        combined_text,
        top_lines,
        bottom_lines,
        pairs: all_pairs,
        detected_order,
    }
}

/// Picks the best target slice from text before the cursor when no text is selected:
/// prefers the current paragraph (after the last newline) if it contains inline annotations,
/// otherwise falls back to the full `text_before_cursor` if it contains inline annotations.
pub fn extract_convertible_target(text_before_cursor: &str) -> Option<TargetSlice> {
    if text_before_cursor.is_empty() {
        return None;
    }
    let last_newline = text_before_cursor.rfind('\n');
    let current_paragraph = match last_newline {
        Some(idx) => &text_before_cursor[idx + 1..],
        None => text_before_cursor,
    };
    if !current_paragraph.is_empty()
        && !convert_kipan_text(current_paragraph, false, "auto")
            .pairs
            .is_empty()
    {
        return Some(TargetSlice {
            delete_before_utf16_len: current_paragraph.encode_utf16().count(),
            text: current_paragraph.to_string(),
        });
    }
    if last_newline.is_some()
        && !convert_kipan_text(text_before_cursor, false, "auto")
            .pairs
            .is_empty()
    {
        return Some(TargetSlice {
            delete_before_utf16_len: text_before_cursor.encode_utf16().count(),
            text: text_before_cursor.to_string(),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_example_han_top() {
        let input = "𠊎 (Ngǎi) 無 (mǒ) 幾愛 (gì oi) 客委會 (Hàg-Ví-Fi) 認 (ngin) 證辭彙 (ziin cǐifi)裡肚 (dídù)有兜仔 (iúdéu'è)个 (ge)例 (li) 句 (gi)會 (voi)講 (gòng)體 (tì) 罰 (fad)、權 (kiǎn) 威 (ví)教育 (gau'iùg)个 (ge)感覺 (gàmgòg)";
        let res = convert_kipan_text(input, false, "auto");
        assert_eq!(res.detected_order, "han-top");
        assert_eq!(
            res.top_text,
            "𠊎無幾愛客委會認證辭彙裡肚有兜仔个例句會講體罰、權威教育个感覺"
        );
        assert_eq!(
            res.bottom_text,
            "Ngǎi mǒ gì oi Hàg-Ví-Fi ngin ziin cǐifi dídù iúdéu'è ge li gi voi gòng tì fad, kiǎn ví gau'iùg ge gàmgòg"
        );
    }

    #[test]
    fn manual_force_order_swaps_rows() {
        let input = "𠊎 (Ngǎi) 無 (mǒ) 幾愛 (gì oi) 客委會 (Hàg-Ví-Fi)";
        let res_rom = convert_kipan_text(input, false, "rom-top");
        assert_eq!(res_rom.detected_order, "rom-top");
        assert_eq!(res_rom.top_text, "Ngǎi mǒ gì oi Hàg-Ví-Fi");
        assert_eq!(res_rom.bottom_text, "𠊎無幾愛客委會");

        let res_han = convert_kipan_text(input, false, "han-top");
        assert_eq!(res_han.detected_order, "han-top");
        assert_eq!(res_han.top_text, "𠊎無幾愛客委會");
        assert_eq!(res_han.bottom_text, "Ngǎi mǒ gì oi Hàg-Ví-Fi");
    }

    #[test]
    fn punctuation_bidirectional_conversion() {
        let han_punct = "「食飽、無，恁好。敢係？當好！」（四縣）【美濃】｛註｝";
        assert_eq!(
            convert_punct_to_romance(han_punct),
            "\"食飽,無,恁好.敢係?當好!\"(四縣)[美濃]{註}"
        );

        let rom_punct = "Ngǎi, ngi. he? ho! (Sixian) [Meinung] {Note}";
        assert_eq!(
            convert_punct_to_han(rom_punct),
            "Ngǎi， ngi。 he？ ho！ （Sixian） ［Meinung］ ｛Note｝"
        );

        let input = "試驗 (Cii-ngiàm)：好 (hò)！【認證】 (Ngin-ziin)……";
        let res = convert_kipan_text(input, false, "auto");
        assert_eq!(res.top_text, "試驗：好！【認證】……");
        assert_eq!(res.bottom_text, "Cii-ngiàm: hò! [Ngin-ziin]...");
    }

    #[test]
    fn extended_punctuation_bidirectional_conversion() {
        assert_eq!(
            convert_punct_to_romance("—～〜·‧・‥⋯⋯⋯／＼＋＝＜＞＆％＃＠＊＿｜｀＄＾"),
            "-~~.........../\\+=<>&%#@*_|`$^"
        );
        assert_eq!(
            convert_punct_to_romance("《客話》〈四縣〉〔註〕〖補〗〝引〞．﹗﹖﹔﹕︰"),
            "\"客話\"'四縣'[註][補]\"引\".!?;::"
        );
        assert_eq!(
            convert_punct_to_han("~/\\+=&%#@*_|$^"),
            "～／＼＋＝＆％＃＠＊＿｜＄＾"
        );

        let input = "《客話 (Hakfa)》 ～ 〈四縣 (Sixian)〉 — 試驗 (Cii-ngiàm)／對照 (Dui-zàu)·註 (Zi)";
        let res = convert_kipan_text(input, false, "auto");
        assert_eq!(res.top_text, "《客話》～〈四縣〉—試驗／對照·註");
        assert_eq!(
            res.bottom_text,
            "\"Hakfa\"~'Sixian'-Cii-ngiàm/Dui-zàu. Zi"
        );
    }

    #[test]
    fn reverse_case_roman_first_automatically_rom_top() {
        let input = "Ngǎi (𠊎) mǒ (無) gì oi (幾愛) Hàg-Ví-Fi (客委會)！";
        let res = convert_kipan_text(input, false, "auto");
        assert_eq!(res.detected_order, "rom-top");
        assert_eq!(res.top_text, "Ngǎi mǒ gì oi Hàg-Ví-Fi!");
        assert_eq!(res.bottom_text, "𠊎無幾愛客委會！");
    }

    #[test]
    fn brackets_and_ruby_compatibility() {
        let full = convert_kipan_text("客源翠（Hàg-Ngiǎn-Cui） 企盤（Kì-Pǎn）", false, "auto");
        assert_eq!(full.top_text, "客源翠企盤");
        assert_eq!(full.bottom_text, "Hàg-Ngiǎn-Cui Kì-Pǎn");

        let sq = convert_kipan_text("無成功 [mǒ sǐin'gúng] 無反應 [mǒ fàn'in]", false, "auto");
        assert_eq!(sq.combined_text, "無成功無反應\nmǒ sǐin'gúng mǒ fàn'in");

        let curly = convert_kipan_text("試驗 {Cii-ngiàm} 好 {hò}", false, "auto");
        assert_eq!(curly.top_text, "試驗好");
        assert_eq!(curly.bottom_text, "Cii-ngiàm hò");

        let ruby = convert_kipan_text(
            "<ruby>𠊎<rt>Ngǎi</rt></ruby><ruby>識<rt>sii</rt></ruby><ruby>講<rt>gòng</rt></ruby><ruby>客<rt>Hàg</rt></ruby>。",
            false,
            "auto",
        );
        assert_eq!(ruby.combined_text, "𠊎識講客。\nNgǎi sii gòng Hàg.");
    }

    #[test]
    fn neutral_version_preserved_in_both_lines() {
        let input = "這幾下日 (ià giá ngìd) 做到恁仔 (zo do àn'é)，有想愛 (iú xiòng'oi) 推到 (túi do) v0.0.5 吔 (ě)";
        let res = convert_kipan_text(input, false, "auto");
        assert_eq!(res.top_text, "這幾下日做到恁仔，有想愛推到 v0.0.5 吔");
        assert_eq!(
            res.bottom_text,
            "ià giá ngìd zo do àn'é, iú xiòng'oi túi do v0.0.5 ě"
        );
    }

    #[test]
    fn english_words_preserved_in_han_outside_all_positions() {
        // Start, middle, with punctuation, and sentence end
        let input = "Python 係 (he) 𠊎 (Ngǎi) 用 (iung) Rust，寫 (xià) HakSpring-Kipan。";
        let res = convert_kipan_text(input, false, "auto");
        assert_eq!(res.detected_order, "han-top");
        assert_eq!(res.top_text, "Python 係𠊎用 Rust，寫 HakSpring-Kipan。");
        assert_eq!(
            res.bottom_text,
            "Python he Ngǎi iung Rust, xià HakSpring-Kipan."
        );

        // Sentence end without punctuation
        let tail = convert_kipan_text("推到 (túi do) v0.0.5", false, "auto");
        assert_eq!(tail.top_text, "推到 v0.0.5");
        assert_eq!(tail.bottom_text, "túi do v0.0.5");
    }

    #[test]
    fn english_words_preserved_in_rom_outside_all_positions() {
        // Start, middle, end, numbers, and coincidental ASCII English words (`Go`)
        let input = "Python he (係) Ngǎi (𠊎) iung (用) Rust, xià (寫) iPhone 16 hò (好) v0.0.5!";
        let res = convert_kipan_text(input, false, "auto");
        assert_eq!(res.detected_order, "rom-top");
        assert_eq!(
            res.top_text,
            "Python he Ngǎi iung Rust, xià iPhone 16 hò v0.0.5!"
        );
        assert_eq!(
            res.bottom_text,
            "Python 係𠊎用 Rust，寫 iPhone 16 好 v0.0.5！"
        );

        // Ensure normal Hakfa multi-word romanization is NOT split
        let hakfa_multi = convert_kipan_text(
            "Ngǎi (𠊎) mǒ (無) gì oi (幾愛) ziin cǐifi (證辭彙) túi do (推到) v0.0.5 ě (吔)",
            false,
            "auto",
        );
        assert_eq!(hakfa_multi.detected_order, "rom-top");
        assert_eq!(
            hakfa_multi.top_text,
            "Ngǎi mǒ gì oi ziin cǐifi túi do v0.0.5 ě"
        );
        assert_eq!(
            hakfa_multi.bottom_text,
            "𠊎無幾愛證辭彙推到 v0.0.5 吔"
        );
    }

    #[test]
    fn punctuation_alignment_not_misdetected_by_prefix_or_previous_line() {
        // Prefix Hanji before `Rom (Han)` must not flip `is_han_outside` to true
        let prefix_han = convert_kipan_text("試驗：Ngǎi (𠊎) mǒ (無)。", false, "auto");
        assert_eq!(prefix_han.detected_order, "rom-top");
        assert_eq!(prefix_han.top_text, "試驗: Ngǎi mǒ.");
        assert_eq!(prefix_han.bottom_text, "試驗：𠊎無。");

        // Multi-line `rom-top` followed by neutral-only line keeps rom-top punctuation alignment
        let multi_rom = convert_kipan_text("Ngǎi (𠊎)。\nOK (OK)。", false, "auto");
        assert_eq!(multi_rom.detected_order, "rom-top");
        assert_eq!(multi_rom.top_text, "Ngǎi.\nOK.");
        assert_eq!(multi_rom.bottom_text, "𠊎。\nOK。");
    }

    #[test]
    fn multi_paragraphs_preserves_line_breaks() {
        let input = "𠊎 (Ngǎi) 識 (sii) 講 (gòng) 客 (Hàg)。\n汝 (Ngi) 呢 (nē)？";
        let res = convert_kipan_text(input, false, "auto");
        assert_eq!(res.top_text, "𠊎識講客。\n汝呢？");
        assert_eq!(res.bottom_text, "Ngǎi sii gòng Hàg.\nNgi nē?");
    }

    #[test]
    fn extract_target_utf16_length_handles_supplementary_hanji() {
        let before = "第一行無標註\n𠊎 (Ngǎi) 講 (gòng) 客 (Hàg)";
        let target = extract_convertible_target(before).expect("should find target");
        assert_eq!(target.text, "𠊎 (Ngǎi) 講 (gòng) 客 (Hàg)");
        assert_eq!(
            target.delete_before_utf16_len,
            "𠊎 (Ngǎi) 講 (gòng) 客 (Hàg)".encode_utf16().count()
        );

        let trailing_nl = "𠊎 (Ngǎi) 講 (gòng) 客 (Hàg)\n";
        let target2 = extract_convertible_target(trailing_nl).expect("should fallback");
        assert_eq!(target2.text, trailing_nl);
        assert_eq!(target2.delete_before_utf16_len, trailing_nl.encode_utf16().count());

        assert!(extract_convertible_target("").is_none());
        assert!(extract_convertible_target("純粹漢字無標註").is_none());
    }
}
