//! Hakfa fork: Meinong tones end to end — the production dictionary, the real fetch. Tone 24 is
//! written as 33 in the display romanization only. Its own process (the engine handles are
//! process-wide). Skips when `assets/dictionaries/` is not built.

mod common;

use protos::engine::{
    composing_request, request, response, AppConfig, Append, CandidateMessage, ComposingRequest,
    DictionarySourceToggles, FetchAtPos,
};

fn candidates(config: AppConfig, keys: &[&str], generation: u64) -> Vec<CandidateMessage> {
    let send = |method| {
        common::roundtrip(
            config.clone(),
            generation,
            request::Payload::Composing(ComposingRequest { method: Some(method) }),
        )
    };
    for key in keys {
        send(composing_request::Method::Append(Append { char: (*key).to_owned() }));
    }
    let toggles = DictionarySourceToggles {
        kautian: true,
        taigitv: true,
        kungge: true,
        lect: config.lect,
        meinong: config.meinong_tones,
        ..DictionarySourceToggles::default()
    };
    match send(composing_request::Method::FetchAtPos(FetchAtPos {
        toggles: Some(toggles),
        ..FetchAtPos::default()
    }))
    .payload
    {
        Some(response::Payload::Composing(composing)) => {
            composing.continuous.expect("candidates").candidates
        }
        other => panic!("expected a composing payload, got {other:?}"),
    }
}

#[test]
fn tone_24_is_written_as_33_only_when_asked_and_the_identity_stays_24() {
    if !common::production_lexicon_ready() {
        eprintln!("skip: no production dictionary");
        return;
    }
    // 厥 reads giá (tone 24) in 四縣.
    let plain = AppConfig { input_mode: "tl".into(), lect: 1, ..AppConfig::default() };
    let ordinary = candidates(plain.clone(), &["g", "i", "a"], 1);
    let giá = ordinary
        .iter()
        .find(|c| c.hanji.as_deref() == Some("厥") && c.canonical_tl == "giá")
        .expect("四縣 offers 厥 giá");
    assert_eq!(giá.roman, "giá");

    let meinong = AppConfig { meinong_tones: true, ..plain };
    let tones = candidates(meinong, &["g", "i", "a"], 2);
    let giā = tones
        .iter()
        .find(|c| c.hanji.as_deref() == Some("厥") && c.canonical_tl == "giá")
        .expect("still the same word");
    assert_eq!(giā.roman, "giā", "written as tone 33");
    assert_eq!(giā.canonical_tl, "giá", "the identity is untouched");
    assert_eq!(giā.display_text, giá.display_text, "the commit text is untouched");
}
