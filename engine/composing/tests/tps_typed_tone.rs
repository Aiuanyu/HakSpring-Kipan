//! §17 case 4 — TPS typed tone marks and spaces pin the candidate readings.
//!
//! Reported (2026-10-10 TPS real-text sim, Windows + Android): a span with
//! SOME syllables marked took the toneless key and nothing re-applied the
//! marks, so `ㄉㄞ˫ㄨㄢ` composed 台灣 `tâi-uân` (tone 5) over the typed
//! tone 7, and a mark or a space did not stop the §35 `ㄫ`→`ㄥ` reading from
//! pulling the next onset into the previous syllable — `ㄌㄧˋㄫㆤ˫` composed
//! 靈會 `lîng ē` instead of 你硬 `lí ngē`.
//!
//! After the fix every TPS candidate source checks the reading against the
//! typed closes: a mark's syllable must end there with that tone, an
//! interior space must be a syllable end of any tone, and the §41 trailing
//! space keeps tone 1/4.
//!
//! Fixture shape follows `tps_space_pinned_tone.rs` (hermetic
//! `LexiconHandle`, both `tps:` key families). Per the fixture-coverage rule
//! in `docs/contributing/known-pitfalls.md`, the strict-prefix production
//! syllables of the probed keys are present and asserted on: 有 `ū` (ㄨ)
//! under ㄨㄢ. The coda-steal shapes (`ㄌㄧˋㄫㆤ˫`, `ㄌㄧ`␣`ㄫㆤ`) need the
//! production syllable inventory to reproduce, so they live in
//! `walker_fixed_inputs_prod.rs`.

use crate::common::{
    build_dictionary_fst_tps, build_syllables_fst_tps, build_tkdb_v4, empty_association_bin,
    fetch_hanji_with_custom, install_lexicon, Row,
};
use crate::tps_hanji_conversion::row;
use lexicon::CustomEntry;
use test_support::{engine_install_lock, write_temp};

/// The wrong reading is always the more frequent one, so only the pin can
/// put the typed word first.
fn fixture_rows() -> Vec<Row> {
    vec![
        row("台灣", "tâi-uân", 2, 1000),
        row("大員", "tāi-uân", 2, 10),
        row("台", "tâi", 1, 500),
        row("大", "tāi", 1, 400),
        row("員", "uân", 1, 100),
        row("有", "ū", 1, 300),
        row("毋是", "m̄-sī", 2, 80),
        row("是", "sī", 1, 600),
    ]
}

fn install_fixture_tps() {
    let rows = fixture_rows();
    let dict_path = write_temp("dictionary-tps-typed.bin", &build_tkdb_v4(&rows));
    let fst_path = build_dictionary_fst_tps(&rows);
    let association_path = write_temp("association-tps-typed.bin", &empty_association_bin());
    let syllables_path = build_syllables_fst_tps(&rows);
    install_lexicon(&fst_path, &dict_path, &association_path, &syllables_path);
}

fn fetch_hanji(raw: &str) -> Vec<String> {
    fetch_hanji_with_custom(raw, "tps", Vec::new())
}

fn has(list: &[String], hanji: &str) -> bool {
    list.iter().any(|h| h == hanji)
}

// trace: `ㄉㄞ˫ㄨㄢ` → toneless key `tps:ㄉㄞㄨㄢ`, pin Marked(7) at 6.
// 台灣 tâi-uân: tai5 ✗; 大員 tāi-uân ✓; 台 tâi ✗ / 大 tāi ✓ on the
// one-syllable span; 有 ū is the ㄨ strict-prefix control (not a full span).
#[test]
fn partial_mark_keeps_the_typed_tone() {
    let _guard = engine_install_lock();
    install_fixture_tps();
    let list = fetch_hanji("ㄉㄞ˫ㄨㄢ");
    assert_eq!(list.first().map(String::as_str), Some("大員"), "{list:?}");
    assert!(!has(&list, "台灣"), "{list:?}");
    assert!(!has(&list, "台"), "{list:?}");
    assert!(has(&list, "大"), "{list:?}");
    assert!(!has(&list, "有"), "{list:?}");
}

// Control: the toneless buffer still surfaces every tone, frequency first.
#[test]
fn toneless_buffer_still_surfaces_every_tone() {
    let _guard = engine_install_lock();
    install_fixture_tps();
    let list = fetch_hanji("ㄉㄞㄨㄢ");
    assert_eq!(list.first().map(String::as_str), Some("台灣"), "{list:?}");
    assert!(has(&list, "大員"), "{list:?}");
}

// trace: whole-buffer custom merge aligns on the typed body: `ㄉㄞㄨㄢ`
// with Marked(7) at 6 keeps a custom 大員 tāi-uân, drops 台灣 tâi-uân.
#[test]
fn custom_entries_answer_to_the_typed_mark() {
    let _guard = engine_install_lock();
    install_fixture_tps();
    let custom = |roman: &str, hanji: &str| CustomEntry {
        roman: roman.to_owned(),
        hanji: Some(hanji.to_owned()),
    };
    let list = fetch_hanji_with_custom(
        "ㄉㄞ˫ㄨㄢ",
        "tps",
        vec![custom("tâi-uân", "臺灣"), custom("tāi-uân", "大灣")],
    );
    assert!(!has(&list, "臺灣"), "{list:?}");
    assert!(has(&list, "大灣"), "{list:?}");
}

// trace: partial-prefix extension of `ㄇ˫ㄒ` — §35 reaches `tps:ㆬㄒㄧ`
// 毋是 m̄-sī; the pin aligns on that MATCHED key (`ㆬ` ends at 3, tone 7),
// the typed body's literal `ㄇ` would reject it.
#[test]
fn partial_prefix_extension_aligns_on_the_substituted_key() {
    let _guard = engine_install_lock();
    install_fixture_tps();
    let list = fetch_hanji("ㄇ˫ㄒ");
    assert!(has(&list, "毋是"), "{list:?}");
}
