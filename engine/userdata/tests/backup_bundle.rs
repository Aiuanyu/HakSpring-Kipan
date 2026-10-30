//! Hakfa fork: the all-lects bundle (version 4) — every lect's learning, learned phrases
//! included, and the custom dictionary with each row's lect, in one `.hakfa` file.

use protos::engine::{
    user_data_request::Method, user_data_response::Result as Answer, ExportBackup, ImportBackup,
    OpenUserData, UserDataJournal, UserDataRequest,
};
use userdata::{is_bundle, CustomDictionaryRow, UserDataHandle, BUNDLE_VERSION};

fn open(directory: &std::path::Path) -> UserDataHandle {
    let handle = UserDataHandle::new();
    handle
        .handle(&UserDataRequest {
            method: Some(Method::Open(OpenUserData {
                directory: directory.display().to_string(),
                journal: UserDataJournal::Delete as i32,
                ..OpenUserData::default()
            })),
        })
        .unwrap();
    handle
}

fn export(handle: &UserDataHandle) -> Vec<u8> {
    let response = handle
        .handle(&UserDataRequest {
            method: Some(Method::ExportBackup(ExportBackup {
                platform: "test".into(),
                app_version: "0".into(),
                all_lects: true,
                ..ExportBackup::default()
            })),
        })
        .unwrap();
    match response.result {
        Some(Answer::BackupExported(exported)) => exported.backup,
        other => panic!("expected BackupExported, got {other:?}"),
    }
}

fn import(handle: &UserDataHandle, backup: Vec<u8>) -> protos::engine::BackupImported {
    let response = handle
        .handle(&UserDataRequest {
            method: Some(Method::ImportBackup(ImportBackup {
                backup,
                ..ImportBackup::default()
            })),
        })
        .unwrap();
    match response.result {
        Some(Answer::BackupImported(imported)) => imported,
        other => panic!("expected BackupImported, got {other:?}"),
    }
}

#[test]
fn a_bundle_carries_every_lects_learning_and_the_custom_dictionary_with_its_lects() {
    let from = tempfile::tempdir().unwrap();
    let source = open(from.path());
    let sixian = source.stores_for_lect(1).unwrap();
    let hailu = source.stores_for_lect(2).unwrap();
    let nam = source.stores_for_lect(6).unwrap();
    sixian.frequency.record("海", "hói");
    hailu.frequency.record("海", "hói");
    hailu.frequency.record("海", "hói");
    nam.frequency.record("你", "ngí");
    hailu.learned_phrases.learn_phrase("海洋", "hói-yòng");
    hailu.learned_phrases.learn_phrase("海洋", "hói-yòng");
    sixian
        .custom_dictionary
        .import_until_full(&[
            CustomDictionaryRow::new_with_lect("vì", "瑋", 1),
            CustomDictionaryRow::new_with_lect("vi", "瑋", 2),
        ])
        .unwrap();

    let file = export(&source);
    assert!(
        is_bundle(&file),
        "a bundle is recognized by its version and lects"
    );
    let text = String::from_utf8(file.clone()).unwrap();
    assert!(text.contains(&format!("\"version\": {BUNDLE_VERSION}")));
    assert!(text.contains("learnedPhrases"));

    let to = tempfile::tempdir().unwrap();
    let target = open(to.path());
    let imported = import(&target, file);
    assert_eq!(imported.refusal, protos::engine::BackupRefusal::None as i32);
    assert_eq!(imported.lects, 3, "四縣, 海陸, 南四縣 sections");
    assert_eq!(imported.custom_dictionary, 2);
    assert_eq!(imported.learned_phrases, 1);

    let hailu = target.stores_for_lect(2).unwrap();
    assert_eq!(hailu.frequency.all_rows().unwrap().len(), 1);
    assert_eq!(
        hailu.frequency.all_rows().unwrap()[0].count,
        2,
        "the count survives"
    );
    assert_eq!(hailu.learned_phrases.all_rows().unwrap().len(), 1);
    assert_eq!(hailu.learned_phrases.all_rows().unwrap()[0].learn_count, 2);
    assert!(
        target
            .stores_for_lect(1)
            .unwrap()
            .learned_phrases
            .all_rows()
            .unwrap()
            .is_empty(),
        "a lect's learned phrases stay in that lect"
    );
    assert_eq!(
        target
            .stores_for_lect(6)
            .unwrap()
            .frequency
            .all_rows()
            .unwrap()
            .len(),
        1
    );
    let mut lects: Vec<u32> = target
        .stores_for_lect(1)
        .unwrap()
        .custom_dictionary
        .all_rows()
        .unwrap()
        .into_iter()
        .map(|row| row.lect)
        .collect();
    lects.sort_unstable();
    assert_eq!(lects, vec![1, 2], "each custom row keeps its lect");
}

#[test]
fn importing_a_bundle_twice_adds_nothing_and_keeps_the_larger_counts() {
    let from = tempfile::tempdir().unwrap();
    let source = open(from.path());
    let hailu = source.stores_for_lect(2).unwrap();
    hailu.frequency.record("海", "hói");
    hailu.learned_phrases.learn_phrase("海洋", "hói-yòng");
    let file = export(&source);

    let to = tempfile::tempdir().unwrap();
    let target = open(to.path());
    import(&target, file.clone());
    let again = import(&target, file);
    assert_eq!(again.custom_dictionary, 0);

    let hailu = target.stores_for_lect(2).unwrap();
    assert_eq!(hailu.frequency.all_rows().unwrap().len(), 1);
    assert_eq!(hailu.learned_phrases.all_rows().unwrap().len(), 1);
    assert_eq!(
        hailu.learned_phrases.all_rows().unwrap()[0].learn_count,
        1,
        "not summed"
    );
}

#[test]
fn a_single_lect_backup_is_not_a_bundle() {
    assert!(!is_bundle(br#"{"version": 3, "userFrequency": []}"#));
    assert!(!is_bundle(b"not json"));
}

/// A real bundle passes 2 MiB (the FFI request cap), so a platform sends it in parts.
#[test]
fn a_bundle_sent_in_parts_imports_like_one_sent_whole() {
    let from = tempfile::tempdir().unwrap();
    let source = open(from.path());
    let hailu = source.stores_for_lect(2).unwrap();
    hailu.frequency.record("海", "hói");
    hailu.learned_phrases.learn_phrase("海洋", "hói-yòng");
    let file = export(&source);
    assert!(file.len() > 300, "enough bytes to split");

    let to = tempfile::tempdir().unwrap();
    let target = open(to.path());
    let parts: Vec<&[u8]> = file.chunks(97).collect();
    let last = parts.len() - 1;
    let mut answer = None;
    for (index, part) in parts.iter().enumerate() {
        let response = target
            .handle(&UserDataRequest {
                method: Some(Method::ImportBackup(ImportBackup {
                    backup: part.to_vec(),
                    part: index as u32,
                    more: index != last,
                    ..ImportBackup::default()
                })),
            })
            .unwrap();
        match response.result {
            Some(Answer::BackupImported(imported)) if index == last => answer = Some(imported),
            Some(Answer::BackupImported(imported)) => {
                assert_eq!(
                    imported.lects, 0,
                    "a part before the last imports nothing yet"
                );
            }
            other => panic!("expected BackupImported, got {other:?}"),
        }
    }
    let imported = answer.unwrap();
    assert_eq!(imported.refusal, protos::engine::BackupRefusal::None as i32);
    assert_eq!(imported.lects, 3);
    assert_eq!(imported.learned_phrases, 1);
    assert_eq!(
        target
            .stores_for_lect(2)
            .unwrap()
            .learned_phrases
            .all_rows()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn a_new_part_zero_discards_what_an_abandoned_import_left() {
    let to = tempfile::tempdir().unwrap();
    let target = open(to.path());
    let send = |part: u32, more: bool, bytes: &[u8]| {
        target
            .handle(&UserDataRequest {
                method: Some(Method::ImportBackup(ImportBackup {
                    backup: bytes.to_vec(),
                    part,
                    more,
                    ..ImportBackup::default()
                })),
            })
            .unwrap()
    };
    send(0, true, b"{\"junk\":");
    send(1, true, b"[1,2");
    // an abandoned import; a fresh part 0 starts clean
    let from = tempfile::tempdir().unwrap();
    let file = export(&open(from.path()));
    let response = send(0, false, &file);
    match response.result {
        Some(Answer::BackupImported(imported)) => {
            assert_eq!(imported.refusal, protos::engine::BackupRefusal::None as i32);
            assert_eq!(imported.lects, 3);
        }
        other => panic!("expected BackupImported, got {other:?}"),
    }
}
