//! Hakfa fork: 各腔互學 (Obs IME data-ui D3). What was learned in one lect is offered to the
//! others, where the target lect's dictionary reads the same Hanji in exactly one way.
//!
//! The rule is the maintainer's (USER 2026-10-25): take what the source lects hold; if the target
//! lect has no such Hanji, or no single reading for it, skip. Nothing is guessed, so a polyphonic
//! word (the target reads it several ways) is never copied into the wrong reading. A copy counts
//! once and never raises what the target lect learned on its own.
//!
//! The dictionary lookup is a parameter, so the algorithm runs against a stub in the tests and
//! against `lexicon::api::search_by_hanji` in the engine.

use std::collections::HashSet;
use std::sync::Arc;

use userdata::{UserDataHandle, UserDataStores};

/// How many skipped words the answer lists, for the preview to show why.
const SKIP_EXAMPLES: usize = 12;

/// Why a word was not copied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SkipReason {
    Missing,
    Ambiguous,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SyncReport {
    pub added_phrases: u32,
    pub added_words: u32,
    pub already_present: u32,
    pub skipped_missing: u32,
    pub skipped_ambiguous: u32,
    pub skipped_examples: Vec<(String, u32, SkipReason)>,
}

impl SyncReport {
    fn skip(&mut self, hanji: &str, target: u32, reason: SkipReason) {
        match reason {
            SkipReason::Missing => self.skipped_missing += 1,
            SkipReason::Ambiguous => self.skipped_ambiguous += 1,
        }
        if self.skipped_examples.len() < SKIP_EXAMPLES {
            self.skipped_examples.push((hanji.to_owned(), target, reason));
        }
    }
}

/// What a lect's dictionary says about a Hanji: its distinct readings there.
pub(crate) type Readings<'a> = dyn Fn(&str, u32) -> Vec<String> + 'a;

/// What the target lect would do with one source row.
enum Outcome {
    Copy(String),
    Skip(SkipReason),
}

fn outcome(readings: &Readings<'_>, hanji: &str, target: u32) -> Outcome {
    let mut distinct: Vec<String> = Vec::new();
    for reading in readings(hanji, target) {
        if !distinct.contains(&reading) {
            distinct.push(reading);
        }
    }
    match distinct.len() {
        0 => Outcome::Skip(SkipReason::Missing),
        1 => Outcome::Copy(distinct.remove(0)),
        _ => Outcome::Skip(SkipReason::Ambiguous),
    }
}

/// Evens out the learning data of `lects` (lect codes): every lect's learned phrases and word
/// counts are offered to every other one of them. `dry_run` counts and writes nothing.
pub(crate) fn sync(
    handle: &UserDataHandle,
    lects: &[u32],
    dry_run: bool,
    readings: &Readings<'_>,
) -> Option<SyncReport> {
    let stores: Vec<(u32, Arc<UserDataStores>)> = lects
        .iter()
        .map(|lect| handle.stores_for_lect(*lect).map(|stores| (*lect, stores)))
        .collect::<Option<_>>()?;
    for (_, set) in &stores {
        set.frequency.open_blocking();
        set.learned_phrases.open_blocking();
    }
    // Everything is read before anything is written, so a copy made on the way never becomes a
    // source of its own (a word would otherwise echo back to the lect it came from).
    struct Learned {
        phrases: Vec<userdata::LearnedPhraseRow>,
        words: Vec<userdata::FrequencyRow>,
        have_phrases: HashSet<(String, String)>,
        have_words: HashSet<(String, String)>,
    }
    let learned: Vec<(u32, Learned)> = stores
        .iter()
        .map(|(lect, set)| {
            let phrases = set.learned_phrases.all_rows().unwrap_or_default();
            let words = set.frequency.all_rows().unwrap_or_default();
            let have_phrases = phrases
                .iter()
                .map(|row| (row.hanji.clone(), row.canonical_tl.clone()))
                .collect();
            let have_words = words
                .iter()
                .map(|row| (row.word.clone(), row.tl.clone()))
                .collect();
            (*lect, Learned { phrases, words, have_phrases, have_words })
        })
        .collect();

    let mut report = SyncReport::default();
    for (source, from) in &learned {
        for (target, to) in learned.iter().filter(|(lect, _)| lect != source) {
            let Some((_, target_stores)) = stores.iter().find(|(lect, _)| lect == target) else {
                continue;
            };
            for row in &from.phrases {
                match outcome(readings, &row.hanji, *target) {
                    Outcome::Skip(reason) => report.skip(&row.hanji, *target, reason),
                    Outcome::Copy(tl) if to.have_phrases.contains(&(row.hanji.clone(), tl.clone())) => {
                        report.already_present += 1;
                    }
                    Outcome::Copy(tl) => {
                        report.added_phrases += 1;
                        if !dry_run {
                            target_stores.learned_phrases.learn_phrase(&row.hanji, &tl);
                        }
                    }
                }
            }
            for row in &from.words {
                match outcome(readings, &row.word, *target) {
                    Outcome::Skip(reason) => report.skip(&row.word, *target, reason),
                    Outcome::Copy(tl) if to.have_words.contains(&(row.word.clone(), tl.clone())) => {
                        report.already_present += 1;
                    }
                    Outcome::Copy(tl) => {
                        report.added_words += 1;
                        if !dry_run {
                            target_stores.frequency.record(&row.word, &tl);
                        }
                    }
                }
            }
        }
    }
    Some(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use protos::engine::{user_data_request, OpenUserData, UserDataJournal, UserDataRequest};
    use std::collections::HashMap;

    /// The two lects of the fixture: 1 = 四縣, 2 = 海陸.
    fn dictionary() -> HashMap<(&'static str, u32), Vec<&'static str>> {
        HashMap::from([
            (("海", 1), vec!["hòi"]),
            (("海", 2), vec!["hói"]),
            (("海陸", 2), vec!["hói-liug"]),
            (("行", 1), vec!["hàng", "hâng"]), // polyphonic in 四縣
            (("行", 2), vec!["hàng"]),
            (("四縣", 1), vec!["xi-ien"]),
        ])
    }

    fn readings_from(dict: HashMap<(&'static str, u32), Vec<&'static str>>) -> impl Fn(&str, u32) -> Vec<String> {
        move |hanji, lect| {
            dict.iter()
                .find(|((word, l), _)| *word == hanji && *l == lect)
                .map(|(_, tls)| tls.iter().map(|tl| (*tl).to_owned()).collect())
                .unwrap_or_default()
        }
    }

    fn opened_handle(directory: &std::path::Path) -> UserDataHandle {
        let handle = UserDataHandle::new();
        handle
            .handle(&UserDataRequest {
                method: Some(user_data_request::Method::Open(OpenUserData {
                    directory: directory.display().to_string(),
                    journal: UserDataJournal::Delete as i32,
                    ..OpenUserData::default()
                })),
            })
            .unwrap();
        handle
    }

    #[test]
    fn a_word_goes_across_when_the_target_reads_it_one_way_and_is_skipped_otherwise() {
        let directory = tempfile::tempdir().unwrap();
        let handle = opened_handle(directory.path());
        let sixian = handle.stores_for_lect(1).unwrap();
        sixian.frequency.record("海", "hòi");
        sixian.frequency.record("四縣", "xi-ien"); // 海陸 has no such word
        sixian.frequency.record("行", "hàng"); // 海陸 reads it one way: copied
        sixian.learned_phrases.learn_phrase("海陸", "hòi-liùg");
        let readings = readings_from(dictionary());

        let report = sync(&handle, &[1, 2], false, &readings).unwrap();

        let hailu = handle.stores_for_lect(2).unwrap();
        let words: Vec<_> = hailu.frequency.all_rows().unwrap().into_iter().map(|r| (r.word, r.tl)).collect();
        assert!(words.contains(&("海".into(), "hói".into())), "海陸 reading, not the 四縣 one: {words:?}");
        assert!(words.contains(&("行".into(), "hàng".into())));
        assert!(!words.iter().any(|(word, _)| word == "四縣"), "no such word in 海陸");
        let phrases: Vec<_> = hailu.learned_phrases.all_rows().unwrap().into_iter().map(|r| (r.hanji, r.canonical_tl)).collect();
        assert_eq!(phrases, vec![("海陸".to_owned(), "hói-liug".to_owned())]);
        assert_eq!((report.added_words, report.added_phrases), (2, 1));
        assert_eq!(report.skipped_missing, 1);
        assert_eq!(report.skipped_examples, vec![("四縣".into(), 2, SkipReason::Missing)]);
    }

    #[test]
    fn a_polyphonic_target_is_never_guessed() {
        let directory = tempfile::tempdir().unwrap();
        let handle = opened_handle(directory.path());
        let hailu = handle.stores_for_lect(2).unwrap();
        hailu.frequency.record("行", "hàng");
        let readings = readings_from(dictionary());

        // 海陸 → 四縣: 四縣 reads 行 two ways, so it is skipped.
        let report = sync(&handle, &[1, 2], false, &readings).unwrap();

        assert_eq!(report.skipped_ambiguous, 1);
        assert_eq!(report.added_words, 0);
        assert!(handle.stores_for_lect(1).unwrap().frequency.all_rows().unwrap().is_empty());
    }

    #[test]
    fn a_dry_run_counts_and_writes_nothing() {
        let directory = tempfile::tempdir().unwrap();
        let handle = opened_handle(directory.path());
        handle.stores_for_lect(1).unwrap().frequency.record("海", "hòi");
        let readings = readings_from(dictionary());

        let preview = sync(&handle, &[1, 2], true, &readings).unwrap();

        assert_eq!(preview.added_words, 1);
        assert!(handle.stores_for_lect(2).unwrap().frequency.all_rows().unwrap().is_empty());
        // the real run then does what the preview said
        let done = sync(&handle, &[1, 2], false, &readings).unwrap();
        assert_eq!(done.added_words, 1);
        assert_eq!(handle.stores_for_lect(2).unwrap().frequency.all_rows().unwrap().len(), 1);
    }

    #[test]
    fn what_the_target_has_is_left_alone_and_a_second_run_adds_nothing() {
        let directory = tempfile::tempdir().unwrap();
        let handle = opened_handle(directory.path());
        handle.stores_for_lect(1).unwrap().frequency.record("海", "hòi");
        let hailu = handle.stores_for_lect(2).unwrap();
        for _ in 0..5 {
            hailu.frequency.record("海", "hói");
        }
        let readings = readings_from(dictionary());

        let first = sync(&handle, &[1, 2], false, &readings).unwrap();
        let second = sync(&handle, &[1, 2], false, &readings).unwrap();

        assert_eq!(first.added_words, 0);
        assert!(first.already_present >= 1);
        assert_eq!(second.added_words, 0);
        let row = hailu.frequency.all_rows().unwrap().into_iter().find(|r| r.word == "海").unwrap();
        assert_eq!(row.count, 5, "the target's own count is not raised");
    }
}
