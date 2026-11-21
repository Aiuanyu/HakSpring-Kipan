//! Traces per-keystroke continuous candidates from the real Kipan engine for demo videos.

use std::env;
use std::path::{Path, PathBuf};
use prost::Message;
use protos::engine::{
    composing_request, request, response, user_data_request, AppConfig, Append, ComposingRequest,
    DictionarySourceToggles, FetchAtPos, OpenUserData, Request, Response, Start,
    SyllableSeparator, UserDataJournal, UserDataRequest,
};
use userdata::{JournalMode, UserDataPaths, UserDataStores};

fn roundtrip(config: &AppConfig, generation: u64, payload: request::Payload) -> Response {
    let req = Request {
        id: 1,
        config_snapshot: Some(config.clone()),
        generation,
        payload: Some(payload),
    };
    Response::decode(dispatch::process_request(&req.encode_to_vec()).as_slice())
        .expect("decode response")
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut input = String::from("Iazagqiutienn'gamvoixiongoihiKiangjiangced");
    let mut lect: u32 = 1;
    let mut out_path = PathBuf::from("out/trace.json");
    let mut learned: Vec<(String, String)> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" if i + 1 < args.len() => {
                input = args[i + 1].clone();
                i += 2;
            }
            "--lect" if i + 1 < args.len() => {
                lect = args[i + 1].parse().expect("valid --lect integer");
                i += 2;
            }
            "--out" if i + 1 < args.len() => {
                out_path = PathBuf::from(&args[i + 1]);
                i += 2;
            }
            "--learn" if i + 1 < args.len() => {
                if let Some((hanji, rom)) = args[i + 1].split_once('=') {
                    learned.push((hanji.to_owned(), rom.to_owned()));
                }
                i += 2;
            }
            other => panic!("Unknown argument: {other}"),
        }
    }

    let root = repo_root();
    let dict_dir = root.join("assets/dictionaries");
    let paths = lexicon::LexiconPaths::validated(
        dict_dir.join("dictionary.fst").to_str().unwrap(),
        dict_dir.join("dictionary.bin").to_str().unwrap(),
        dict_dir.join("association.bin").to_str().unwrap(),
        dict_dir.join("syllables.fst").to_str().unwrap(),
        0,
    )
    .expect("validate lexicon paths");
    lexicon::EngineHandle::install(paths).expect("install lexicon");

    let config = AppConfig {
        input_mode: "tl".into(),
        is_hanji_first: true,
        syllable_separator: SyllableSeparator::None as i32,
        lect,
        ..AppConfig::default()
    };
    let toggles = DictionarySourceToggles {
        kautian: true,
        taigitv: true,
        kungge: true,
        lect,
        ..DictionarySourceToggles::default()
    };

    if !learned.is_empty() {
        let ud_dir = env::temp_dir().join("kipan_demo_trace_ud");
        let _ = std::fs::remove_dir_all(&ud_dir);
        std::fs::create_dir_all(&ud_dir).unwrap();
        {
            let stores =
                UserDataStores::at(UserDataPaths::in_directory(&ud_dir), JournalMode::Delete);
            stores.open_blocking();
            for (hanji, rom) in &learned {
                stores.learned_phrases.learn_phrase(hanji, rom);
            }
            stores.learned_phrases.all_rows();
        }
        roundtrip(
            &config,
            2,
            request::Payload::UserData(UserDataRequest {
                method: Some(user_data_request::Method::Open(OpenUserData {
                    directory: ud_dir.display().to_string(),
                    journal: UserDataJournal::Delete as i32,
                    ..OpenUserData::default()
                })),
            }),
        );
    }

    roundtrip(
        &config,
        2,
        request::Payload::Composing(ComposingRequest {
            method: Some(composing_request::Method::Start(Start::default())),
        }),
    );

    let mut acc = String::new();
    let mut items = Vec::new();
    for ch in input.chars() {
        acc.push(ch);
        roundtrip(
            &config,
            2,
            request::Payload::Composing(ComposingRequest {
                method: Some(composing_request::Method::Append(Append {
                    char: ch.to_string(),
                })),
            }),
        );
        let resp = roundtrip(
            &config,
            2,
            request::Payload::Composing(ComposingRequest {
                method: Some(composing_request::Method::FetchAtPos(FetchAtPos {
                    toggles: Some(toggles.clone()),
                    ..FetchAtPos::default()
                })),
            }),
        );
        if let Some(response::Payload::Composing(comp)) = resp.payload {
            let cands = comp.continuous.map(|c| c.candidates).unwrap_or_default();
            let mut cand_json = Vec::new();
            for c in cands.iter().filter(|c| c.hanji.is_some()).take(9) {
                let han = c.hanji.as_deref().unwrap_or("");
                cand_json.push(format!(
                    "{{\"han\": {}, \"rom\": {}}}",
                    json_escape(han),
                    json_escape(&c.roman)
                ));
            }
            items.push(format!(
                "  {{\"preedit\": {}, \"cands\": [{}]}}",
                json_escape(&acc),
                cand_json.join(", ")
            ));
        }
    }

    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).unwrap();
        }
    }
    let json = format!("[\n{}\n]\n", items.join(",\n"));
    std::fs::write(&out_path, json).expect("write trace json");
    println!("Wrote {} steps to {}", items.len(), out_path.display());
}
