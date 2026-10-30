//! Hakfa fork: a whole-lect backup bundle is bigger than the FFI request cap (2 MiB), so it is
//! imported in parts. The cap lives in the FFI adapters (`dispatch::is_request_too_large`); this
//! pins that a real-sized bundle needs the parts and that each part passes the gate.
//! Its own process: the user-data handle is process-wide.
#![cfg(feature = "user-data")]

mod common;

use common::{open_user_data, tl_config, user_data};
use prost::Message;
use protos::engine::{
    request, response, user_data_request, user_data_response, ExportBackup, ImportBackup, Request,
    UserDataRequest,
};
use userdata::UserDataHandle;

const PART: usize = 1024 * 1024;

fn encoded_len(method: &user_data_request::Method) -> usize {
    Request {
        id: 1,
        config_snapshot: Some(tl_config(false)),
        generation: 0,
        payload: Some(request::Payload::UserData(UserDataRequest {
            method: Some(method.clone()),
        })),
    }
    .encoded_len()
}

fn imported(response: protos::engine::Response) -> protos::engine::BackupImported {
    match response.payload {
        Some(response::Payload::UserData(data)) => match data.result {
            Some(user_data_response::Result::BackupImported(imported)) => imported,
            other => panic!("expected BackupImported, got {other:?}"),
        },
        other => panic!("expected user data, got {other:?}"),
    }
}

#[test]
fn a_real_sized_bundle_needs_parts_and_every_part_passes_the_ffi_gate() {
    let directory = tempfile::tempdir().unwrap();
    open_user_data(directory.path());
    let hailu = UserDataHandle::instance().stores_for_lect(2).unwrap();
    hailu.frequency.open_blocking();
    let rows: Vec<(String, String, i64)> = (0..20_000)
        .map(|n| (format!("字{n}"), format!("ji{n}"), 3))
        .collect();
    hailu.frequency.import_merge(rows).unwrap();

    let exported = user_data(user_data_request::Method::ExportBackup(ExportBackup {
        platform: "test".into(),
        app_version: "0".into(),
        all_lects: true,
        ..ExportBackup::default()
    }));
    let file = match exported.payload {
        Some(response::Payload::UserData(data)) => match data.result {
            Some(user_data_response::Result::BackupExported(exported)) => exported.backup,
            other => panic!("expected BackupExported, got {other:?}"),
        },
        other => panic!("expected user data, got {other:?}"),
    };

    let whole = user_data_request::Method::ImportBackup(ImportBackup {
        backup: file.clone(),
        ..ImportBackup::default()
    });
    assert!(
        dispatch::is_request_too_large(encoded_len(&whole)),
        "the whole file ({} bytes) is over the FFI cap — the reason imports go in parts",
        file.len()
    );

    let parts: Vec<&[u8]> = file.chunks(PART).collect();
    let last = parts.len() - 1;
    let mut answer = None;
    for (index, part) in parts.iter().enumerate() {
        let method = user_data_request::Method::ImportBackup(ImportBackup {
            backup: part.to_vec(),
            part: index as u32,
            more: index != last,
            ..ImportBackup::default()
        });
        assert!(
            !dispatch::is_request_too_large(encoded_len(&method)),
            "part {index} passes the FFI gate"
        );
        let result = imported(user_data(method));
        if index == last {
            answer = Some(result);
        }
    }
    let result = answer.unwrap();
    assert_eq!(result.refusal, protos::engine::BackupRefusal::None as i32);
    assert_eq!(result.lects, 3);
    assert!(result.frequency >= 20_000);
}
