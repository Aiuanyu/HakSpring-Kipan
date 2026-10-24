//! The user-data stores, opened once per process on the platform's word
//! (`UserDataRequest.open`) and reset on its word (`.reset`). What they hold
//! is the engine's (`docs/contributing/rust-migration-policy.md` §6); where the
//! files live is the platform's. Plan: `docs/architecture/user-data-engine-roadmap.md`.

use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Once, OnceLock};

use protos::engine::{
    user_data_request, user_data_response, OpenUserData, UsageRecorded, UserDataJournal,
    UserDataOpened, UserDataRequest, UserDataResponse,
};

use crate::{JournalMode, RequestError, UserDataPaths, UserDataStores};

/// Where and how the stores were opened, and the stores.
struct Opened {
    paths: UserDataPaths,
    journal: JournalMode,
    stores: Arc<UserDataStores>,
    /// The open finishing — files open, custom dictionary taken over — run
    /// once however many callers wait on it, on whichever
    /// thread gets there first.
    initialized: Arc<Once>,
    /// Hakfa fork: the learning stores of the lects other than 四縣, opened on first use
    /// (`stores_for_lect`). 四縣 is `stores` itself.
    lects: Mutex<HashMap<u32, Arc<UserDataStores>>>,
}

/// The stores this process opened. Set once, then read without a lock: from
/// roadmap P3b every keystroke reads through [`UserDataHandle::stores`], and
/// must never wait behind an open (its takeover can take seconds).
pub struct UserDataHandle {
    opened: OnceLock<Opened>,
}

impl UserDataHandle {
    /// A handle of its own, for tests that must not share the process one (Hakfa fork: public for
    /// `engine/dispatch`'s lect-sync tests).
    pub const fn new() -> Self {
        Self {
            opened: OnceLock::new(),
        }
    }

    /// The process's one handle.
    pub fn instance() -> &'static Self {
        static HANDLE: UserDataHandle = UserDataHandle::new();
        &HANDLE
    }

    /// Answers one user-data request: `Open`, `RecordUsage`, or a page request.
    pub fn handle(&self, request: &UserDataRequest) -> Result<UserDataResponse, RequestError> {
        let method = request
            .method
            .as_ref()
            .ok_or(RequestError::Invalid("user-data request has no method"))?;
        let result = match method {
            user_data_request::Method::Open(open) => {
                user_data_response::Result::Opened(self.open(open)?)
            }
            user_data_request::Method::RecordUsage(usage) => {
                self.record_usage(usage)?;
                user_data_response::Result::UsageRecorded(UsageRecorded {})
            }
            // Hakfa fork: how much each lect has learned.
            user_data_request::Method::CountLearning(count) => {
                user_data_response::Result::LearningCounts(self.count_learning(count)?)
            }
            // Hakfa fork: a page acts on the learning stores of the lect it names.
            page => {
                let stores = self.settled_stores_for_lect(requested_lect(page))?;
                Self::handle_page(&stores, page)?
            }
        };
        Ok(UserDataResponse {
            result: Some(result),
        })
    }

    /// The open stores, for the engine's own reads and writes; `None` until
    /// the platform opens them. Never blocks.
    pub fn stores(&self) -> Option<&UserDataStores> {
        self.opened.get().map(|opened| &*opened.stores)
    }

    /// Hakfa fork: the stores `lect` learns into. 0 and 1 (四縣) are the base set, with the
    /// file names that existed before lects; any other lect gets its own frequency,
    /// association and learned-phrase files (`UserDataPaths::for_lect`), created and opened on
    /// first use, sharing the one custom dictionary. A learned word of one lect is invisible
    /// to the others. `None` until the platform opens the stores. Never blocks (a first use
    /// queues its writes behind the file's open, like the base set does).
    pub fn stores_for_lect(&self, lect: u32) -> Option<Arc<UserDataStores>> {
        let opened = self.opened.get()?;
        if crate::paths::lect_file_suffix(lect).is_none() {
            return Some(Arc::clone(&opened.stores));
        }
        let mut lects = opened.lects.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let stores = lects.entry(lect).or_insert_with(|| {
            let stores = opened
                .stores
                .for_lect(opened.paths.for_lect(lect), opened.journal);
            stores.open_learning();
            Arc::new(stores)
        });
        Some(Arc::clone(stores))
    }

    fn open(&self, open: &OpenUserData) -> Result<UserDataOpened, RequestError> {
        let paths = requested_paths(open)?;
        let journal = journal(open.journal());
        // The stores are published before they finish opening: from here on
        // a write queues behind the open on its store's worker and a read
        // answers neutral until the file is ready, so once this call has
        // been handled nothing a caller reports is lost.
        let opened = self.opened.get_or_init(|| {
            let stores = UserDataStores::at(paths.clone(), journal);
            stores.open();
            Opened {
                paths: paths.clone(),
                journal,
                stores: Arc::new(stores),
                initialized: Arc::new(Once::new()),
                lects: Mutex::new(HashMap::new()),
            }
        });
        if opened.paths != paths || opened.journal != journal {
            return Err(RequestError::Invalid(
                "user data is already open at other paths or with another journal",
            ));
        }
        if open.in_background {
            let (stores, initialized) =
                (Arc::clone(&opened.stores), Arc::clone(&opened.initialized));
            let spawned = std::thread::Builder::new()
                .name("taigi-user-data-open".into())
                .spawn(move || finish_open(&stores, &initialized));
            if let Err(error) = spawned {
                // No thread to spare: finish here rather than never.
                log::error!("user_data.open_thread_failed error={error}");
                finish_open(&opened.stores, &opened.initialized);
            }
        } else {
            finish_open(&opened.stores, &opened.initialized);
        }
        Ok(UserDataOpened {})
    }

    /// The open stores, or the refusal a request before the open gets. Never
    /// blocks: for `RecordUsage`, sent from the key path, whose write queues
    /// behind the open.
    pub(crate) fn opened_stores(&self) -> Result<&UserDataStores, RequestError> {
        self.stores()
            .ok_or(RequestError::Invalid("user data is not open yet"))
    }

    /// Hakfa fork: [`Self::settled_stores`] for `lect`: the base set for 四縣 (lect 0 or 1), or that
    /// lect's own learning files, opened here and now (a page may read them at once).
    fn settled_stores_for_lect(&self, lect: u32) -> Result<Arc<UserDataStores>, RequestError> {
        self.settled_stores()?;
        let stores = self
            .stores_for_lect(lect)
            .ok_or(RequestError::Invalid("user data is not open yet"))?;
        stores.frequency.open_blocking();
        stores.association.open_blocking();
        stores.learned_phrases.open_blocking();
        Ok(stores)
    }

    /// Hakfa fork: the rows of each lect's three learning stores.
    fn count_learning(
        &self,
        count: &protos::engine::CountLearning,
    ) -> Result<protos::engine::LearningCounts, RequestError> {
        let lects: Vec<u32> = if count.lects.is_empty() {
            vec![1, 6, 2]
        } else {
            count.lects.clone()
        };
        let mut counts = Vec::new();
        for lect in lects {
            let stores = self.settled_stores_for_lect(lect)?;
            let rows = |len: Option<usize>| u32::try_from(len.unwrap_or(0)).unwrap_or(u32::MAX);
            counts.push(protos::engine::LectLearningCount {
                lect,
                frequency: rows(stores.frequency.all_rows().map(|r| r.len())),
                learned_phrases: rows(stores.learned_phrases.all_rows().map(|r| r.len())),
                association: rows(stores.association.all_rows().map(|r| r.len())),
            });
        }
        Ok(protos::engine::LearningCounts { lects: counts })
    }

    /// The open stores once they have finished opening — for the pages'
    /// requests (`requests.rs::handle_page`), which run off the key path and must
    /// not read or edit a custom dictionary still being taken over or re-derived
    /// by a background open. Waits for that open, or finishes it here.
    fn settled_stores(&self) -> Result<&UserDataStores, RequestError> {
        let opened = self
            .opened
            .get()
            .ok_or(RequestError::Invalid("user data is not open yet"))?;
        finish_open(&opened.stores, &opened.initialized);
        Ok(&opened.stores)
    }
}

/// Hakfa fork: the lect a page request names (0 when it names none: 四縣).
fn requested_lect(method: &user_data_request::Method) -> u32 {
    use user_data_request::Method;
    match method {
        Method::Reset(reset) => reset.lect,
        Method::ExportBackup(export) => export.lect,
        Method::ImportBackup(import) => import.lect,
        Method::ListLearningRecords(list) => list.lect,
        Method::SetLearningRecordCount(set) => set.lect,
        Method::DeleteLearningRecord(delete) => delete.lect,
        Method::AddLearningRecordToCustomDictionary(add) => add.lect,
        _ => 0,
    }
}

/// Waits for every store to open and finishes the custom dictionary's
/// takeover, once per process.
fn finish_open(stores: &UserDataStores, initialized: &Once) {
    initialized.call_once(|| {
        stores.open_blocking();
        log::info!(
            "user_data.open frequency={} association={} custom_dictionary={} learned_phrases={}",
            stores.frequency.is_ready(),
            stores.association.is_ready(),
            stores.custom_dictionary.is_ready(),
            stores.learned_phrases.is_ready()
        );
    });
}

/// The files `open` names: its directory under the shared names, with a
/// non-empty `association_path` overriding that one file (Android).
fn requested_paths(open: &OpenUserData) -> Result<UserDataPaths, RequestError> {
    let mut paths = UserDataPaths::in_directory(&absolute(&open.directory)?);
    if !open.association_path.is_empty() {
        paths.association = absolute(&open.association_path)?;
    }
    Ok(paths)
}

fn absolute(path: &str) -> Result<PathBuf, RequestError> {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        Ok(path)
    } else {
        Err(RequestError::Invalid(
            "user-data paths must be absolute and non-empty",
        ))
    }
}

fn journal(journal: UserDataJournal) -> JournalMode {
    match journal {
        UserDataJournal::Wal => JournalMode::Wal,
        UserDataJournal::Delete => JournalMode::Delete,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn open_request(directory: &std::path::Path) -> UserDataRequest {
        UserDataRequest {
            method: Some(user_data_request::Method::Open(OpenUserData {
                directory: directory.display().to_string(),
                journal: UserDataJournal::Delete as i32,
                ..OpenUserData::default()
            })),
        }
    }

    pub(crate) fn opened(response: UserDataResponse) -> UserDataOpened {
        match response.result {
            Some(user_data_response::Result::Opened(opened)) => opened,
            other => panic!("expected Opened, got {other:?}"),
        }
    }

    #[test]
    fn open_readies_every_store_and_leaves_the_custom_dictionary_empty() {
        let directory = tempfile::tempdir().unwrap();
        let handle = UserDataHandle::new();

        opened(handle.handle(&open_request(directory.path())).unwrap());

        let stores = handle.stores().unwrap();
        assert!(stores.frequency.is_ready());
        assert!(stores.association.is_ready());
        assert!(stores.custom_dictionary.is_ready());
        assert!(stores.learned_phrases.is_ready());
        assert_eq!(
            stores.custom_dictionary.count().unwrap(),
            0,
            "a fresh install starts with no custom words"
        );
    }

    #[test]
    fn a_repeat_open_at_the_same_paths_answers_and_other_paths_are_refused() {
        let directory = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let handle = UserDataHandle::new();
        handle.handle(&open_request(directory.path())).unwrap();

        assert!(handle.handle(&open_request(directory.path())).is_ok());
        assert_eq!(
            handle.handle(&open_request(elsewhere.path())).unwrap_err(),
            RequestError::Invalid(
                "user data is already open at other paths or with another journal"
            )
        );
    }

    #[test]
    fn a_repeat_open_with_another_journal_is_refused() {
        let directory = tempfile::tempdir().unwrap();
        let handle = UserDataHandle::new();
        handle.handle(&open_request(directory.path())).unwrap();
        let mut wal = open_request(directory.path());
        if let Some(user_data_request::Method::Open(open)) = wal.method.as_mut() {
            open.journal = UserDataJournal::Wal as i32;
        }

        assert!(matches!(handle.handle(&wal), Err(RequestError::Invalid(_))));
    }

    #[test]
    fn a_relative_or_empty_directory_is_refused_before_anything_opens() {
        for directory in ["", "relative/dir"] {
            let handle = UserDataHandle::new();
            let request = open_request(std::path::Path::new(directory));

            assert!(matches!(
                handle.handle(&request),
                Err(RequestError::Invalid(_))
            ));
            assert!(handle.stores().is_none());
        }
    }

    #[test]
    fn a_relative_association_override_is_refused_before_anything_opens() {
        let handle = UserDataHandle::new();
        let mut request = open_request(std::path::Path::new("/tmp"));
        if let Some(user_data_request::Method::Open(open)) = request.method.as_mut() {
            open.association_path = "user_association.db".into();
        }

        assert!(matches!(
            handle.handle(&request),
            Err(RequestError::Invalid(_))
        ));
        assert!(handle.stores().is_none());
    }

    #[test]
    fn an_association_override_replaces_that_one_file() {
        // trace: Android keeps `user_association.db` in `filesDir`, the other
        // three stores in `directory`.
        let directory = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        let association = elsewhere.path().join("user_association.db");
        let handle = UserDataHandle::new();
        let mut request = open_request(directory.path());
        if let Some(user_data_request::Method::Open(open)) = request.method.as_mut() {
            open.association_path = association.display().to_string();
        }

        opened(handle.handle(&request).unwrap());

        let stores = handle.stores().unwrap();
        assert!(stores.association.is_ready());
        assert!(association.exists(), "the override names the file");
        assert!(
            !directory.path().join("user_association.db").exists(),
            "and the directory's own slot stays empty"
        );
    }

    #[test]
    fn lect_zero_and_one_are_the_base_stores_and_another_lect_has_its_own_files() {
        // Hakfa fork: what 四縣 learned before lects existed stays 四縣's (same files), and
        // 海陸 learns into files of its own, so neither sees the other's words.
        let directory = tempfile::tempdir().unwrap();
        let handle = UserDataHandle::new();
        assert!(handle.stores_for_lect(2).is_none(), "nothing before the open");
        opened(handle.handle(&open_request(directory.path())).unwrap());

        let sixian = handle.stores_for_lect(1).unwrap();
        assert!(Arc::ptr_eq(&sixian, &handle.stores_for_lect(0).unwrap()));
        let hailu = handle.stores_for_lect(2).unwrap();
        assert!(!Arc::ptr_eq(&sixian, &hailu));
        assert!(
            Arc::ptr_eq(&hailu, &handle.stores_for_lect(2).unwrap()),
            "opened once, kept"
        );
        assert!(Arc::ptr_eq(&sixian.custom_dictionary, &hailu.custom_dictionary));

        sixian.frequency.record("海陸", "hòi-liùg");
        sixian.learned_phrases.learn_phrase("海陸", "hòi-liùg");
        hailu.frequency.record("海", "hói");
        assert_eq!(sixian.frequency.all_rows().unwrap().len(), 1);
        assert_eq!(hailu.frequency.all_rows().unwrap().len(), 1);
        assert_eq!(hailu.frequency.all_rows().unwrap()[0].word, "海");
        assert_eq!(sixian.learned_phrases.all_rows().unwrap().len(), 1);
        assert!(hailu.learned_phrases.all_rows().unwrap().is_empty());

        for name in ["user_frequency.db", "user_frequency.hoiliug.db", "learned_phrases.hoiliug.db", "user_association.hoiliug.db"] {
            assert!(directory.path().join(name).exists(), "{name}");
        }
        assert!(!directory.path().join("custom_dictionary.hoiliug.db").exists());
    }

    fn answer(handle: &UserDataHandle, method: user_data_request::Method) -> user_data_response::Result {
        handle
            .handle(&UserDataRequest { method: Some(method) })
            .unwrap()
            .result
            .unwrap()
    }

    #[test]
    fn a_page_acts_on_the_stores_of_the_lect_it_names() {
        // Hakfa fork (D5): list, count and clear per lect; the original files are 四縣's.
        use protos::engine::{
            CountLearning, LearningRecordKind, ListLearningRecords, ResetUserData,
        };
        let directory = tempfile::tempdir().unwrap();
        let handle = UserDataHandle::new();
        opened(handle.handle(&open_request(directory.path())).unwrap());
        handle.stores_for_lect(1).unwrap().frequency.record("海", "hòi");
        handle.stores_for_lect(1).unwrap().frequency.record("山", "sân");
        handle.stores_for_lect(2).unwrap().frequency.record("海", "hói");

        let list = |lect| {
            match answer(
                &handle,
                user_data_request::Method::ListLearningRecords(ListLearningRecords {
                    kind: LearningRecordKind::Frequency as i32,
                    limit: 50,
                    lect,
                    ..ListLearningRecords::default()
                }),
            ) {
                user_data_response::Result::LearningRecords(records) => records
                    .records
                    .into_iter()
                    .map(|record| (record.text, record.tl))
                    .collect::<Vec<_>>(),
                other => panic!("expected records, got {other:?}"),
            }
        };
        assert_eq!(list(0).len(), 2, "no lect named = 四縣");
        assert_eq!(list(1), list(0));
        assert_eq!(list(2), vec![("海".to_owned(), "hói".to_owned())]);
        assert!(list(6).is_empty(), "南四縣 has learned nothing yet");

        let counts = match answer(&handle, user_data_request::Method::CountLearning(CountLearning::default())) {
            user_data_response::Result::LearningCounts(counts) => counts,
            other => panic!("expected counts, got {other:?}"),
        };
        let frequency: Vec<(u32, u32)> = counts.lects.iter().map(|c| (c.lect, c.frequency)).collect();
        assert_eq!(frequency, vec![(1, 2), (6, 0), (2, 1)]);

        // clearing 海陸 leaves 四縣 alone
        answer(
            &handle,
            user_data_request::Method::Reset(ResetUserData {
                frequency: true,
                lect: 2,
                ..ResetUserData::default()
            }),
        );
        assert!(list(2).is_empty());
        assert_eq!(list(1).len(), 2);
    }
}
