//! Hakfa fork: 各腔互學 end to end — the production merged dictionary and real stores in a
//! temporary directory, through the same request the settings page sends. Its own process (the
//! user-data handle is process-wide). Skips when `assets/dictionaries/` is not built.
#![cfg(feature = "user-data")]

mod common;

use protos::engine::{user_data_request, user_data_response, response, SyncLects};
use userdata::UserDataHandle;

fn sync(dry_run: bool) -> protos::engine::LectsSynced {
    let answer = common::user_data(user_data_request::Method::SyncLects(SyncLects {
        lects: vec![1, 2],
        dry_run,
    }));
    match answer.payload {
        Some(response::Payload::UserData(data)) => match data.result {
            Some(user_data_response::Result::LectsSynced(synced)) => synced,
            other => panic!("expected LectsSynced, got {other:?}"),
        },
        other => panic!("expected user data, got {other:?}"),
    }
}

#[test]
fn what_one_lect_learned_reaches_the_other_with_the_other_lects_reading() {
    if !common::production_lexicon_ready() {
        eprintln!("skip: no production dictionary");
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    common::open_user_data(directory.path());
    let handle = UserDataHandle::instance();
    let sixian = handle.stores_for_lect(1).unwrap();
    let hailu = handle.stores_for_lect(2).unwrap();
    // 四縣 learned the word 海 with its 四縣 reading.
    sixian.frequency.record("海", "hòi");

    let preview = sync(true);
    assert!(preview.dry_run);
    assert!(preview.added_words >= 1, "{preview:?}");
    hailu.frequency.open_blocking();
    assert!(hailu.frequency.all_rows().unwrap().is_empty(), "a preview writes nothing");

    let done = sync(false);
    assert_eq!(done.added_words, preview.added_words);
    let rows = hailu.frequency.all_rows().unwrap();
    assert!(
        rows.iter().any(|row| row.word == "海" && row.tl.starts_with("hói")),
        "the 海陸 reading of 海, not the 四縣 one: {rows:?}"
    );
    assert!(
        !rows.iter().any(|row| row.word == "海" && row.tl.starts_with("hòi")),
        "no 四縣 reading leaks into 海陸: {rows:?}"
    );

    let again = sync(false);
    assert_eq!(again.added_words, 0, "nothing new the second time: {again:?}");
}
