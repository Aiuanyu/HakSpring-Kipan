//! Hakfa fork: `FetchAtPos.before_caret` — with the caret mid-tail (a `MoveCaret`), the syllable is cut
//! there and only the part before the caret is found again; a pick nails that prefix and the rest stays
//! pending. Without the flag, or with the caret at the tail's end, the whole tail is listed as before.

use composing::api::{CaretDirection, Phase};
use composing::{requests, Intent};
use test_support::{engine_install_lock, write_temp};

use crate::common::{
    build_dictionary_fst, build_syllables_fst, build_tkdb_v4, config_tl, continuous_engine,
    empty_association_bin, install_lexicon, Fetch, Row,
};

fn row(toneless_key: &'static str, hanji: &'static str, tl: &'static str, freq: u32) -> Row {
    Row {
        toneless_key,
        hanji,
        tl,
        syll: tl.split(['-', ' ']).filter(|s| !s.is_empty()).count() as u8,
        freq,
    }
}

fn install_fixture() {
    let rows = vec![
        row("tai", "台", "tâi", 5_000),
        row("gi", "語", "gí", 4_000),
        row("taigi", "台語", "tâi-gí", 4_000),
        row("bun", "文", "bûn", 3_000),
    ];
    let dict_path = write_temp("dictionary.bin", &build_tkdb_v4(&rows));
    let fst_path = build_dictionary_fst(&rows);
    let association_path = write_temp("association.bin", &empty_association_bin());
    let syllables_path = build_syllables_fst(&["tai5", "gi2", "bun5"]);
    install_lexicon(&fst_path, &dict_path, &association_path, &syllables_path);
}

/// `(hanji, consumed span end)` of every candidate the engine lists for `taigibun` with the caret at
/// byte `caret` (`None` = left at the end).
fn listed(caret: Option<usize>, before_caret: bool) -> Vec<(Option<String>, u32)> {
    let config = config_tl();
    let mut engine = continuous_engine(&config, "taigibun");
    if let Some(caret) = caret {
        requests::apply(
            Intent::MoveCaret {
                direction: Some(CaretDirection::Start),
            },
            &mut engine,
            &config,
        );
        for _ in 0..caret {
            requests::apply(
                Intent::MoveCaret {
                    direction: Some(CaretDirection::Right),
                },
                &mut engine,
                &config,
            );
        }
        let Phase::Continuous { caret: at, .. } = engine.snapshot_state().phase else {
            panic!("not composing");
        };
        assert_eq!(at, caret);
    }
    let fetch = Fetch {
        before_caret,
        ..Default::default()
    };
    requests::apply(fetch.intent(), &mut engine, &config)
        .continuous
        .expect("continuous carrier")
        .candidates
        .into_iter()
        .map(|c| (c.hanji, c.consumed_span_end))
        .collect()
}

#[test]
fn with_the_caret_mid_tail_only_the_part_before_it_is_found_again() {
    let _lock = engine_install_lock();
    install_fixture();
    // trace: caret after `taigi` (byte 5): `bun` waits behind it.
    let cells = listed(Some(5), true);
    assert!(
        cells.iter().any(|(h, _)| h.as_deref() == Some("台語")),
        "got {cells:?}"
    );
    assert!(
        cells
            .iter()
            .all(|(h, _)| h.as_deref().is_none_or(|h| !h.contains('文'))),
        "the part after the caret must not be listed: {cells:?}"
    );
    assert!(cells.iter().all(|(_, end)| *end <= 5), "got {cells:?}");
}

#[test]
fn the_syllable_is_cut_at_the_caret() {
    let _lock = engine_install_lock();
    install_fixture();
    // trace: caret after `tai` (byte 3): `gibun` is behind it, so 台語 (which needs the `gi`) is out.
    let cells = listed(Some(3), true);
    assert!(
        cells.iter().any(|(h, _)| h.as_deref() == Some("台")),
        "got {cells:?}"
    );
    assert!(
        cells.iter().all(|(h, _)| h.as_deref() != Some("台語")),
        "got {cells:?}"
    );
}

#[test]
fn without_the_flag_or_with_the_caret_at_the_end_the_whole_tail_is_listed() {
    let _lock = engine_install_lock();
    install_fixture();
    let whole = listed(None, false);
    assert!(
        whole
            .iter()
            .any(|(h, _)| h.as_deref().is_some_and(|h| h.contains('文'))),
        "got {whole:?}"
    );
    assert_eq!(listed(Some(5), false), listed(Some(5), false));
    assert!(
        listed(Some(5), false)
            .iter()
            .any(|(h, _)| h.as_deref().is_some_and(|h| h.contains('文'))),
        "the flag off keeps the whole tail"
    );
    assert_eq!(
        listed(None, true),
        whole,
        "the caret at the end changes nothing"
    );
}

#[test]
fn a_caret_at_the_front_lists_nothing() {
    let _lock = engine_install_lock();
    install_fixture();
    assert!(listed(Some(0), true).is_empty());
}
