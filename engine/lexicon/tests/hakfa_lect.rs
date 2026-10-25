//! Hakfa fork: the lect gate against the real merged dictionary
//! (`node dictionary/sources/official/hakspring/extract_and_build.js --lect all`).
//! Skips when the generated file is absent (it is a build output, not checked in).

use lexicon::dictionary_reader::{DictionaryReader, Filter, WIRE_LECT_SHIFT, WIRE_MEINONG_BIT};

fn merged_dictionary() -> Option<DictionaryReader> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../dictionary/output/hakfa-all/dictionary.bin");
    if !path.exists() {
        eprintln!("skip: {} not built", path.display());
        return None;
    }
    Some(DictionaryReader::open(&path).expect("merged dictionary opens"))
}

/// Source bits 0..=11 all on, no subcollection gating, wanted lect on bits 27-29.
fn filter_for(lect: u32) -> Filter {
    Filter::from_enabled_bitmask(0x0FFF | (1 << 12) | (lect << WIRE_LECT_SHIFT) | WIRE_MEINONG_BIT)
}

/// The same with the Meinong switch off (its default).
fn filter_without_meinong(lect: u32) -> Filter {
    Filter::from_enabled_bitmask(0x0FFF | (1 << 12) | (lect << WIRE_LECT_SHIFT))
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
    let hailu = count_passing(&reader, &filter_for(2));
    let south_sixian = count_passing(&reader, &filter_for(6));
    assert!(sixian > 0 && hailu > 0 && south_sixian > 0, "all three lects present");
    assert_eq!(sixian + hailu + south_sixian, total, "every row belongs to exactly one lect");
    assert_eq!(count_passing(&reader, &filter_for(0)), total, "lect 0 = every lect");
    for unbuilt in [3, 4, 5, 7] {
        assert_eq!(count_passing(&reader, &filter_for(unbuilt)), 0, "lect {unbuilt} is not built yet");
    }
}

#[test]
fn meinong_readings_are_south_sixian_rows_hidden_until_the_switch_is_on() {
    let Some(reader) = merged_dictionary() else { return };
    let with = count_passing(&reader, &filter_for(6));
    let without = count_passing(&reader, &filter_without_meinong(6));
    assert!(with > without, "there are Meinong rows in South Sixian: {with} vs {without}");
    // no other lect has any
    for lect in [1, 2] {
        assert_eq!(count_passing(&reader, &filter_for(lect)), count_passing(&reader, &filter_without_meinong(lect)));
    }
}
