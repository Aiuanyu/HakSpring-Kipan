//! Hakfa fork: a one-letter input in 四縣 still reaches the common words whose key is longer than the
//! shortest ones (`t` → 同 `tung`). The partial-prefix scan (`PARTIAL_PREFIX_SCAN_CAP`) is cut in
//! shortest-key-first order before the frequency pick, so a dictionary that grows past the cap loses
//! these words (20261031: the six-lect rebuild doubled the entries under `t`; 14 of the 26 one-letter
//! inputs lost words, among them 食 under `s`). Over the production lexicon; skips without it.

use crate::common::{fetch_hanji, production_lexicon_ready, Fetch};

/// The device's filter for 四縣: lect 1 on wire bits 27-29, plus 教典 / 認證 / 補字典.
const SIXIAN: u32 = (1 << 27) | 0x43;

fn sixian(raw: &str) -> Vec<String> {
    fetch_hanji(
        raw,
        "tl",
        Fetch {
            enabled_sources_bitmask: SIXIAN,
            ..Fetch::default()
        },
    )
}

#[test]
fn a_common_word_with_a_long_key_survives_the_partial_prefix_scan() {
    if !production_lexicon_ready() {
        eprintln!("production artifacts absent — run `make dict`; skipping.");
        return;
    }
    // (typed letter, word) — each is a frequent 四縣 word whose key is longer than most under that letter.
    let cases = [
        ("t", "同"),
        ("s", "食"),
        ("g", "講"),
        ("k", "客"),
        ("f", "風"),
        ("z", "正"),
        ("b", "放"),
        ("c", "長"),
    ];
    for (raw, word) in cases {
        let hanji = sixian(raw);
        assert!(
            hanji.iter().any(|h| h == word),
            "`{raw}` lost {word}: the scan cap is too small for the dictionary; got {hanji:?}"
        );
    }
}
