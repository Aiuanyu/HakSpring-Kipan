//! The `.hakfa` backup file: the custom dictionary, the frequency counts and
//! the learned bigrams in one JSON document; learned phrases never travel
//! (§50). One codec for every platform (user-data-engine-roadmap P4b); the
//! file picker and share sheet stay with the platform.
//!
//! Versions share one shape and differ in how a restore treats a bigram
//! reading: 1 and 2 (iOS and Android) may carry POJ, so their readings are
//! folded POJ → TL; 3 (this codec) writes the readings the store holds and
//! restores them unchanged — a fold would misread a TL `eng` / `ek` final as
//! POJ (蔣經國 `tsiúnn-keng-kok` → `king`). An older app still reads a
//! version-3 file, folding its readings as it always did.
//!
//! Reading is lenient, as Android's was: a missing field takes its default,
//! an unknown one is ignored, only a version below 1 is refused — so an old
//! backup, or one from the other phone, still restores. Writing is iOS's
//! shape: sorted keys, pretty-printed, `lastUsed` empty, ids and times of
//! custom rows left out. Every struct below declares its fields in the
//! serialized keys' alphabetical order, which IS the sorted-keys output —
//! whatever `serde_json` features the build unifies.

use std::fmt::Display;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::timestamp::format_rfc3339_utc;
use crate::{
    AssociationPair, AssociationRow, CustomDictionaryRow, CustomDictionaryStore, UserDataStores,
};

/// The version this codec writes.
pub const BACKUP_VERSION: i64 = 3;

/// Hakfa fork: the version of the all-lects bundle (`export_bundle`). It carries every lect's
/// learning (learned phrases too) and the custom dictionary with each row's lect.
pub const BUNDLE_VERSION: i64 = 4;

/// The first version whose bigram readings restore unchanged.
const VERBATIM_READINGS_VERSION: i64 = 3;

#[derive(Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Backup {
    app_version: String,
    custom_dictionary: Vec<CustomEntry>,
    exported_at: String,
    platform: String,
    user_association: Vec<AssociationEntry>,
    user_frequency: Vec<FrequencyEntry>,
    version: i64,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct CustomEntry {
    hanzi: String,
    /// The unreleased 2026-09-20 shape tagged learned phrases `origin = 1`;
    /// read to skip them, never written.
    #[serde(skip_serializing_if = "Option::is_none")]
    origin: Option<i64>,
    roman: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct FrequencyEntry {
    count: i64,
    last_used: String,
    /// Absent in a backup written before the `(word, tl)` pair key: the
    /// legacy `""` bucket.
    tl: Option<String>,
    word: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct AssociationEntry {
    count: i64,
    last_used: String,
    next_tl: String,
    next_word: String,
    prev_tl: Option<String>,
    prev_word: String,
}

/// Why a backup could not be restored.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum BackupError {
    #[error("not a .hakfa backup: {0}")]
    Unreadable(String),
    #[error("unsupported backup version {0}")]
    UnsupportedVersion(i64),
    #[error("{0}")]
    Store(String),
}

fn store_error(error: impl Display) -> BackupError {
    BackupError::Store(error.to_string())
}

/// What a restore merged, per store.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackupImported {
    pub custom_dictionary: usize,
    pub frequency: usize,
    pub association: usize,
    /// Hakfa fork: learned phrases merged (bundle only).
    pub learned_phrases: usize,
    /// Hakfa fork: lect sections read (bundle only).
    pub lects: usize,
}

/// The stores as a `.hakfa` file. `platform` / `app_version` name the
/// writer (`ios`, `android`, `macos`, …); `now_seconds` stamps `exportedAt`.
pub fn export_backup(
    stores: &UserDataStores,
    platform: &str,
    app_version: &str,
    now_seconds: u64,
) -> Result<Vec<u8>, BackupError> {
    let custom = stores.custom_dictionary.all_rows().map_err(store_error)?;
    let frequency = stores
        .frequency
        .all_rows()
        .ok_or_else(|| store_error("frequency store not readable"))?;
    let association = stores
        .association
        .all_rows()
        .ok_or_else(|| store_error("association store not readable"))?;
    let backup = Backup {
        app_version: app_version.to_owned(),
        custom_dictionary: custom
            .into_iter()
            .map(|row| CustomEntry {
                hanzi: row.hanji,
                origin: None,
                roman: row.roman,
            })
            .collect(),
        exported_at: format_rfc3339_utc(now_seconds),
        platform: platform.to_owned(),
        user_association: association
            .into_iter()
            .map(|row| AssociationEntry {
                count: row.count,
                last_used: String::new(),
                next_tl: row.pair.next_tl,
                next_word: row.pair.next,
                prev_tl: Some(row.pair.previous_tl),
                prev_word: row.pair.previous,
            })
            .collect(),
        user_frequency: frequency
            .into_iter()
            .map(|row| FrequencyEntry {
                count: row.count,
                last_used: String::new(),
                tl: Some(row.tl),
                word: row.word,
            })
            .collect(),
        version: BACKUP_VERSION,
    };
    serde_json::to_vec_pretty(&backup).map_err(store_error)
}

/// Restores a `.hakfa` file into the stores, merging with what is there:
/// custom words already stored are skipped and the import fills whatever
/// room the dictionary has left; counts keep the larger of the two and count
/// as used now; bigram readings of a version-1 / -2 file are folded POJ →
/// TL (the phones could write POJ there). A custom row missing its
/// romanization or its Hanji is skipped, as Android's restore did; a missing
/// count reads as 1, as Android's did.
pub fn import_backup(stores: &UserDataStores, bytes: &[u8]) -> Result<BackupImported, BackupError> {
    let backup: Backup = serde_json::from_slice(bytes)
        .map_err(|error| BackupError::Unreadable(error.to_string()))?;
    if backup.version < 1 {
        return Err(BackupError::UnsupportedVersion(backup.version));
    }

    let custom: Vec<CustomDictionaryRow> = backup
        .custom_dictionary
        .iter()
        .filter(|entry| entry.origin != Some(1))
        .filter(|entry| !entry.roman.is_empty() && !entry.hanzi.is_empty())
        .map(|entry| CustomDictionaryRow::new(&entry.roman, &entry.hanzi))
        .collect();
    let custom_dictionary = stores
        .custom_dictionary
        .import_until_full(&custom)
        .map_err(store_error)?
        .imported;

    let frequency = stores
        .frequency
        .import_merge(
            backup
                .user_frequency
                .into_iter()
                .map(|entry| (entry.word, entry.tl.unwrap_or_default(), entry.count.max(1)))
                .collect(),
        )
        .map_err(store_error)?;

    let folds_readings = backup.version < VERBATIM_READINGS_VERSION;
    let reading = |stored: String| {
        if folds_readings {
            phonetics::api::poj_display_to_tl_display(&stored)
        } else {
            stored
        }
    };
    let association = stores
        .association
        .import_merge(
            backup
                .user_association
                .into_iter()
                .map(|entry| AssociationRow {
                    pair: AssociationPair {
                        previous: entry.prev_word,
                        previous_tl: reading(entry.prev_tl.unwrap_or_default()),
                        next: entry.next_word,
                        next_tl: reading(entry.next_tl),
                    },
                    count: entry.count.max(1),
                })
                .collect(),
        )
        .map_err(store_error)?;

    Ok(BackupImported {
        custom_dictionary,
        frequency,
        association,
        ..BackupImported::default()
    })
}

// ---- Hakfa fork: the all-lects bundle (version 4) ----

#[derive(Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Bundle {
    app_version: String,
    custom_dictionary: Vec<BundleCustomEntry>,
    exported_at: String,
    lects: Vec<LectSection>,
    platform: String,
    version: i64,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct BundleCustomEntry {
    hanzi: String,
    lect: u32,
    roman: String,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct LectSection {
    lect: u32,
    learned_phrases: Vec<PhraseEntry>,
    user_association: Vec<AssociationEntry>,
    user_frequency: Vec<FrequencyEntry>,
}

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
struct PhraseEntry {
    count: i64,
    hanzi: String,
    tl: String,
}

/// True when `bytes` is a version-4 bundle (a `lects` array and `version` >= 4), so the importer
/// routes it to [`import_bundle`] rather than the single-lect [`import_backup`].
pub fn is_bundle(bytes: &[u8]) -> bool {
    #[derive(Deserialize, Default)]
    #[serde(default)]
    struct Probe {
        version: i64,
        lects: Option<Vec<serde::de::IgnoredAny>>,
    }
    serde_json::from_slice::<Probe>(bytes)
        .map(|probe| probe.version >= BUNDLE_VERSION && probe.lects.is_some())
        .unwrap_or(false)
}

/// Every lect's frequency, association and learned phrases, plus the whole custom dictionary with
/// each row's lect, as one version-4 `.hakfa` document. `lects` pairs a lect code with the stores
/// that lect learns into.
pub fn export_bundle(
    custom_dictionary: &CustomDictionaryStore,
    lects: &[(u32, Arc<UserDataStores>)],
    platform: &str,
    app_version: &str,
    now_seconds: u64,
) -> Result<Vec<u8>, BackupError> {
    let custom = custom_dictionary.all_rows().map_err(store_error)?;
    let mut sections = Vec::with_capacity(lects.len());
    for (lect, stores) in lects {
        let frequency = stores
            .frequency
            .all_rows()
            .ok_or_else(|| store_error("frequency store not readable"))?;
        let association = stores
            .association
            .all_rows()
            .ok_or_else(|| store_error("association store not readable"))?;
        let phrases = stores
            .learned_phrases
            .all_rows()
            .ok_or_else(|| store_error("learned phrase store not readable"))?;
        sections.push(LectSection {
            lect: *lect,
            learned_phrases: phrases
                .into_iter()
                .map(|row| PhraseEntry {
                    count: row.learn_count,
                    hanzi: row.hanji,
                    tl: row.canonical_tl,
                })
                .collect(),
            user_association: association
                .into_iter()
                .map(|row| AssociationEntry {
                    count: row.count,
                    last_used: String::new(),
                    next_tl: row.pair.next_tl,
                    next_word: row.pair.next,
                    prev_tl: Some(row.pair.previous_tl),
                    prev_word: row.pair.previous,
                })
                .collect(),
            user_frequency: frequency
                .into_iter()
                .map(|row| FrequencyEntry {
                    count: row.count,
                    last_used: String::new(),
                    tl: Some(row.tl),
                    word: row.word,
                })
                .collect(),
        });
    }
    let bundle = Bundle {
        app_version: app_version.to_owned(),
        custom_dictionary: custom
            .into_iter()
            .map(|row| BundleCustomEntry {
                hanzi: row.hanji,
                lect: row.lect,
                roman: row.roman,
            })
            .collect(),
        exported_at: format_rfc3339_utc(now_seconds),
        lects: sections,
        platform: platform.to_owned(),
        version: BUNDLE_VERSION,
    };
    serde_json::to_vec_pretty(&bundle).map_err(store_error)
}

/// Restores a version-4 bundle, merging with what is there: the custom dictionary keeps each row's
/// lect; each lect section merges into `stores_for(lect)` (a lect with no stores is skipped), with
/// counts keeping the larger of the two. Readings are restored unchanged.
pub fn import_bundle(
    custom_dictionary: &CustomDictionaryStore,
    stores_for: &dyn Fn(u32) -> Option<Arc<UserDataStores>>,
    bytes: &[u8],
) -> Result<BackupImported, BackupError> {
    let bundle: Bundle = serde_json::from_slice(bytes)
        .map_err(|error| BackupError::Unreadable(error.to_string()))?;
    if bundle.version < BUNDLE_VERSION {
        return Err(BackupError::UnsupportedVersion(bundle.version));
    }
    let custom: Vec<CustomDictionaryRow> = bundle
        .custom_dictionary
        .iter()
        .filter(|entry| !entry.roman.is_empty() && !entry.hanzi.is_empty())
        .map(|entry| CustomDictionaryRow::new_with_lect(&entry.roman, &entry.hanzi, entry.lect))
        .collect();
    let mut merged = BackupImported {
        custom_dictionary: custom_dictionary
            .import_until_full(&custom)
            .map_err(store_error)?
            .imported,
        ..BackupImported::default()
    };
    for section in bundle.lects {
        let Some(stores) = stores_for(section.lect) else {
            continue;
        };
        merged.frequency += stores
            .frequency
            .import_merge(
                section
                    .user_frequency
                    .into_iter()
                    .map(|entry| (entry.word, entry.tl.unwrap_or_default(), entry.count.max(1)))
                    .collect(),
            )
            .map_err(store_error)?;
        merged.association += stores
            .association
            .import_merge(
                section
                    .user_association
                    .into_iter()
                    .map(|entry| AssociationRow {
                        pair: AssociationPair {
                            previous: entry.prev_word,
                            previous_tl: entry.prev_tl.unwrap_or_default(),
                            next: entry.next_word,
                            next_tl: entry.next_tl,
                        },
                        count: entry.count.max(1),
                    })
                    .collect(),
            )
            .map_err(store_error)?;
        merged.learned_phrases += stores
            .learned_phrases
            .import_merge(
                section
                    .learned_phrases
                    .into_iter()
                    .map(|entry| (entry.hanzi, entry.tl, entry.count))
                    .collect(),
            )
            .map_err(store_error)?;
        merged.lects += 1;
    }
    Ok(merged)
}
