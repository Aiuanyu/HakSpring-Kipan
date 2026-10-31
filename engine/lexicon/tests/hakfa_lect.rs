//! Hakfa fork: the lect gate against the real merged dictionary
//! (`node dictionary/sources/official/hakspring/extract_and_build.js --lect all`).
//! Skips when the generated file is absent (it is a build output, not checked in).

use lexicon::dictionary_reader::{
    DictionaryReader, Filter, WIRE_LECT_SHIFT, WIRE_MEINONG_BIT, WIRE_TAOYUAN_BIT,
    WIRE_ZHUOLAN_BIT,
};

fn merged_dictionary() -> Option<DictionaryReader> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../dictionary/output/hakfa-all/dictionary.bin");
    if !path.exists() {
        eprintln!("skip: {} not built", path.display());
        return None;
    }
    Some(DictionaryReader::open(&path).expect("merged dictionary opens"))
}

/// Source bits 0..=11 all on, no subcollection gating, wanted lect on bits 27-29, sub-dialect toggles on.
fn filter_for(lect: u32) -> Filter {
    Filter::from_enabled_bitmask(
        0x0FFF
            | (1 << 12)
            | (lect << WIRE_LECT_SHIFT)
            | WIRE_MEINONG_BIT
            | WIRE_ZHUOLAN_BIT
            | WIRE_TAOYUAN_BIT,
    )
}

/// The same with the Meinong switch off (its default).
fn filter_without_meinong(lect: u32) -> Filter {
    Filter::from_enabled_bitmask(
        0x0FFF | (1 << 12) | (lect << WIRE_LECT_SHIFT) | WIRE_ZHUOLAN_BIT | WIRE_TAOYUAN_BIT,
    )
}

/// The same with the Raoping sub-dialect switches (Zhuolan / Taoyuan) off.
fn filter_without_raoping_variants(lect: u32) -> Filter {
    Filter::from_enabled_bitmask(0x0FFF | (1 << 12) | (lect << WIRE_LECT_SHIFT) | WIRE_MEINONG_BIT)
}

fn count_passing(reader: &DictionaryReader, filter: &Filter) -> u32 {
    (1..=reader.record_count())
        .filter_map(|rowid| reader.record(rowid))
        .filter(|r| DictionaryReader::passes_filter(r.bitmask, r.kautian_subtag, filter))
        .count() as u32
}

#[test]
fn each_lect_sees_only_its_own_rows_and_lect_zero_sees_all() {
    let Some(reader) = merged_dictionary() else { return };
    let total = reader.record_count();
    let sixian = count_passing(&reader, &filter_for(1));
    let south_sixian = count_passing(&reader, &filter_for(6));
    let hailu = count_passing(&reader, &filter_for(2));
    let dapu = count_passing(&reader, &filter_for(3));
    let raoping = count_passing(&reader, &filter_for(4));
    let zhaoan = count_passing(&reader, &filter_for(5));
    assert!(
        sixian > 0 && south_sixian > 0 && hailu > 0 && dapu > 0 && raoping > 0 && zhaoan > 0,
        "all six lects present"
    );
    assert_eq!(
        sixian + south_sixian + hailu + dapu + raoping + zhaoan,
        total,
        "every row belongs to exactly one lect"
    );
    assert_eq!(count_passing(&reader, &filter_for(0)), total, "lect 0 = every lect");
    assert_eq!(count_passing(&reader, &filter_for(7)), 0, "lect 7 is unused");
}

#[test]
fn meinong_readings_are_south_sixian_rows_hidden_until_the_switch_is_on() {
    let Some(reader) = merged_dictionary() else { return };
    let with = count_passing(&reader, &filter_for(6));
    let without = count_passing(&reader, &filter_without_meinong(6));
    assert!(with > without, "there are Meinong rows in South Sixian: {with} vs {without}");
    // no other lect has any
    for lect in [1, 2, 3, 4, 5] {
        assert_eq!(
            count_passing(&reader, &filter_for(lect)),
            count_passing(&reader, &filter_without_meinong(lect))
        );
    }
}

#[test]
fn raoping_sub_dialect_readings_are_hidden_until_zhuolan_or_taoyuan_switch_is_on() {
    let Some(reader) = merged_dictionary() else { return };
    let with = count_passing(&reader, &filter_for(4));
    let without = count_passing(&reader, &filter_without_raoping_variants(4));
    assert!(
        with > without,
        "there are Zhuolan/Taoyuan rows in Raoping: {with} vs {without}"
    );
    for lect in [1, 6, 2, 3, 5] {
        assert_eq!(
            count_passing(&reader, &filter_for(lect)),
            count_passing(&reader, &filter_without_raoping_variants(lect))
        );
    }
}
