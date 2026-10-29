//! The phrases the user composed segment by segment (§50) — learning data
//! in its own `learned_phrases.db`, never the custom dictionary's (USER
//! 2026-09-21). Port of the former Swift `LearnedPhraseStore`; SQL
//! byte-identical.

use crate::custom_dictionary::SearchKeyDeriver;
use crate::sync_ledger;
use crate::database::{
    immediate_transaction, is_taken_over, mark_taken_over, user_version, JournalMode, StoreSchema,
    UserDataDatabase, UserDataDatabaseError,
};
use phonetics::api::CustomSearchKey;
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashMap;
use std::path::PathBuf;

const TABLE_NAME: &str = "learned_phrases";
const SEARCH_KEY_TABLE_NAME: &str = "learned_search_key";
/// `user_version` — Hakfa bump to 2 to rederive search keys for Hakfa initials (like `ch-`).
const SCHEMA_VERSION: i64 = 2;

/// The table as the Learning Records page reads it. A phrase's search keys
/// go with it.
pub(crate) const LEARNING_TABLE: crate::learning_records::Table = crate::learning_records::Table {
    name: TABLE_NAME,
    identity: ["hanzi", "roman"],
    count: "learn_count",
    last_used: "updated_at",
    delete_dependents: Some(delete_search_keys),
};

/// One learned phrase with its count, for the tests: no product surface
/// lists learned phrases.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedPhraseRow {
    pub hanji: String,
    pub canonical_tl: String,
    pub learn_count: i64,
}

/// Keeps the phrases final commits taught, answers the exact whole-buffer query on
/// the keystroke path, and is wiped with the other learning data.
pub struct LearnedPhraseStore {
    pub(crate) database: UserDataDatabase,
    derive_search_keys: SearchKeyDeriver,
    limit: usize,
}

impl LearnedPhraseStore {
    /// Rows kept; past it the fewest-composed, then least recently touched,
    /// row goes (ChiaKey's policy) so a learn never fails. Ported
    /// from iOS `LearnedPhraseRepository.maxEntries` /
    /// Android `LearnedPhraseService.MAX_ENTRIES`.
    pub const MAX_ENTRIES: usize = 2_000;
    /// Largest `learn_count` a row can carry.
    pub const MAX_LEARN_COUNT: i64 = 1_000_000;
    /// Rows per fetch (relaxed to 25 for whole-buffer + sub-span phrase queries).
    pub const KEYSTROKE_LIMIT: usize = 25;

    /// The cap is injectable ONLY so a test can reach it without writing
    /// 2000 rows.
    pub fn new(
        path: PathBuf,
        journal: JournalMode,
        derive_search_keys: SearchKeyDeriver,
        limit: usize,
    ) -> Self {
        Self {
            database: UserDataDatabase::new(
                "LearnedPhraseStore",
                path,
                journal,
                StoreSchema {
                    apply: apply_schema,
                    max_known_version: SCHEMA_VERSION,
                    marks_takeover_on_open: false,
                },
            ),
            derive_search_keys,
            limit,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.database.is_ready()
    }

    pub fn open(&self) {
        self.database.open();
    }

    pub fn open_blocking(&self) {
        self.database.open_blocking();
        if self.is_ready() {
            self.finish_takeover();
        }
    }

    /// Records one phrase a final commit taught: inserts the `(hanji, canonical
    /// TL)` pair or bumps its `learn_count`, in one statement on the
    /// `(hanji, roman)` unique constraint. A fresh row gets its keys and may
    /// evict past the cap; all in one transaction. Best-effort and off the
    /// keystroke path. The keys are derived before the write lock is taken
    /// (an FFI round-trip has no business holding it).
    pub fn learn_phrase(&self, hanji: &str, canonical_tl: &str) {
        if hanji.is_empty() || canonical_tl.is_empty() {
            return;
        }
        let Some(search_keys) =
            (self.derive_search_keys)(canonical_tl).filter(|keys| !keys.is_empty())
        else {
            return;
        };
        let limit = self.limit;
        let hanji = hanji.to_owned();
        let canonical_tl = canonical_tl.to_owned();
        self.database.write(move |connection| {
            immediate_transaction::<_, rusqlite::Error>(connection, |connection| {
                let (row_id, learn_count): (i64, i64) = connection.query_row(
                    &format!(
                        "INSERT INTO {TABLE_NAME} (roman, hanzi, learn_count, updated_at)\nVALUES (?, ?, 1, CURRENT_TIMESTAMP)\nON CONFLICT(hanzi, roman) DO UPDATE SET\n    learn_count = MIN(learn_count + 1, {}),\n    updated_at = CURRENT_TIMESTAMP\nRETURNING id, learn_count;",
                        Self::MAX_LEARN_COUNT
                    ),
                    params![canonical_tl, hanji],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?;
                sync_ledger::claim(connection, &hanji, &canonical_tl)?;
                // `learn_count` reads 1 only for the row this statement just
                // inserted (a bump lands at 2 or more). If already learned,
                // refresh search keys in case phonetic derivation rules evolved.
                if learn_count != 1 {
                    delete_search_keys(connection, row_id)?;
                    write_search_keys(connection, row_id, &search_keys)?;
                    return Ok(());
                }
                write_search_keys(connection, row_id, &search_keys)?;
                evict_past_cap(connection, limit, row_id)
            })
        });
    }

    /// Hakfa fork (各腔互學): writes the phrases the lect sync copied, in one transaction, without
    /// overflowing the async write queue. A pair that is already a row is left alone; a new row is
    /// noted in the sync ledger as a copy, so the sync never offers it on.
    pub fn learn_phrase_copies(&self, pairs: &[(String, String)]) -> Result<(), UserDataDatabaseError> {
        if pairs.is_empty() {
            return Ok(());
        }
        let mut candidates = Vec::with_capacity(pairs.len());
        for (hanji, canonical_tl) in pairs {
            if hanji.is_empty() || canonical_tl.is_empty() {
                continue;
            }
            if let Some(keys) = (self.derive_search_keys)(canonical_tl).filter(|k| !k.is_empty()) {
                candidates.push((hanji.clone(), canonical_tl.clone(), keys));
            }
        }
        if candidates.is_empty() {
            return Ok(());
        }
        let limit = self.limit;
        self.database.perform(move |connection| {
            immediate_transaction::<_, UserDataDatabaseError>(connection, |connection| {
                let mut last_inserted_id = 0i64;
                for (hanzi, canonical_tl, search_keys) in &candidates {
                    let inserted: Option<i64> = connection
                        .query_row(
                            &format!(
                                "INSERT INTO {TABLE_NAME} (roman, hanzi, learn_count, updated_at)\nVALUES (?, ?, 1, CURRENT_TIMESTAMP)\nON CONFLICT(hanzi, roman) DO NOTHING\nRETURNING id;"
                            ),
                            params![canonical_tl, hanzi],
                            |row| row.get(0),
                        )
                        .optional()?;
                    if let Some(row_id) = inserted {
                        write_search_keys(connection, row_id, search_keys)?;
                        sync_ledger::mark_copied(connection, hanzi, canonical_tl)?;
                        last_inserted_id = row_id;
                    }
                }
                if last_inserted_id > 0 {
                    evict_past_cap(connection, limit, last_inserted_id)?;
                }
                Ok(())
            })
        })
    }

    /// The pairs the lect sync copied here (not the user's own evidence).
    pub fn copied_pairs(&self) -> Option<std::collections::HashSet<(String, String)>> {
        self.database.wait_for_queued_writes();
        self.database
            .read(|connection| sync_ledger::read_pairs(connection, sync_ledger::COPY_TABLE))
    }

    /// The pairs the user deleted here: the lect sync does not copy them back.
    pub fn blocked_pairs(&self) -> Option<std::collections::HashSet<(String, String)>> {
        self.database.wait_for_queued_writes();
        self.database
            .read(|connection| sync_ledger::read_pairs(connection, sync_ledger::BLOCK_TABLE))
    }

    /// Bumps a phrase the user just committed as one candidate, so a phrase
    /// that is used stays ahead of the eviction line. No-op for an unknown pair.
    pub fn touch_phrase(&self, hanji: &str, canonical_tl: &str) {
        if hanji.is_empty() || canonical_tl.is_empty() {
            return;
        }
        let hanji = hanji.to_owned();
        let canonical_tl = canonical_tl.to_owned();
        self.database.write(move |connection| {
            connection.execute(
                &format!(
                    "UPDATE {TABLE_NAME}\nSET learn_count = MIN(learn_count + 1, {}), updated_at = CURRENT_TIMESTAMP\nWHERE hanzi = ? AND roman = ?;",
                    Self::MAX_LEARN_COUNT
                ),
                params![hanji, canonical_tl],
            )?;
            sync_ledger::claim(connection, &hanji, &canonical_tl)?;
            Ok(())
        });
    }

    /// The phrases matching `query_key` for `composing::UserRows.learned`.
    ///
    /// Two-phase separate query (R8-22 / C8-13):
    /// 1. Prefix / Exact / Abbrev query: phrases starting at the beginning of the typed buffer
    ///    (`k.key = ?3 OR ?3 LIKE k.key || '%'`) or matching an abbrev.
    ///    Ordered by `learn_count DESC, length(k.key) DESC, updated_at DESC`.
    /// 2. Substring / Infix query: phrases contained within a longer buffer (`instr(?3, k.key) > 0`).
    ///    Ordered by `length(k.key) DESC, learn_count DESC` so multi-syllable phrases are not
    ///    crowded out by single-syllable short words.
    pub fn rows_matching(
        &self,
        query_key: &CustomSearchKey,
        limit: usize,
    ) -> Vec<LearnedPhraseRow> {
        self.database
            .read(|connection| {
                // 1. Prefix / Exact / Abbrev query:
                // Phrases that match the whole input or anchor at the start of a longer buffer
                // (`?3 LIKE k.key || '%'`), or an exact abbrev match. Learned phrases never
                // prefix-extend from an incomplete key.
                let mut prefix_stmt = connection.prepare_cached(&format!(
                    "SELECT p.hanzi, p.roman, p.learn_count\n\
                     FROM {TABLE_NAME} p\n\
                     JOIN {SEARCH_KEY_TABLE_NAME} k ON k.phrase_id = p.id\n\
                     WHERE k.family = ?1\n\
                       AND ((k.form = ?2 AND ?3 LIKE k.key || '%')\n\
                         OR (k.form = 'abbrev' AND k.key = ?3))\n\
                     GROUP BY p.id\n\
                     ORDER BY p.learn_count DESC, length(k.key) DESC, p.updated_at DESC\n\
                     LIMIT ?4;"
                ))?;
                let mut rows: Vec<LearnedPhraseRow> = prefix_stmt
                    .query_map(
                        params![query_key.family, query_key.form, query_key.key, limit as i64],
                        decode_row,
                    )?
                    .collect::<rusqlite::Result<_>>()?;

                if rows.len() < limit {
                    let mut seen: std::collections::HashSet<(String, String)> =
                        rows.iter().map(|r| (r.hanji.clone(), r.canonical_tl.clone())).collect();
                    let remaining = (limit - rows.len()) as i64;
                    // 2. Substring / Infix query.
                    let mut substr_stmt = connection.prepare_cached(&format!(
                        "SELECT p.hanzi, p.roman, p.learn_count\n\
                         FROM {TABLE_NAME} p\n\
                         JOIN {SEARCH_KEY_TABLE_NAME} k ON k.phrase_id = p.id\n\
                         WHERE k.family = ?1\n\
                           AND k.form = ?2\n\
                           AND instr(?3, k.key) > 0\n\
                           AND ?3 NOT LIKE k.key || '%'\n\
                           AND k.key NOT LIKE ?3 || '%'\n\
                         GROUP BY p.id\n\
                         ORDER BY length(k.key) DESC, p.learn_count DESC, p.updated_at DESC\n\
                         LIMIT ?4;"
                    ))?;
                    let substr_rows = substr_stmt.query_map(
                        params![query_key.family, query_key.form, query_key.key, remaining],
                        decode_row,
                    )?;
                    for row in substr_rows {
                        let row = row?;
                        if seen.insert((row.hanji.clone(), row.canonical_tl.clone())) {
                            rows.push(row);
                        }
                    }
                }
                Ok(rows)
            })
            .unwrap_or_default()
    }

    /// Every phrase, most composed first. For the tests — nothing in the
    /// shipped UI lists learned phrases.
    pub fn all_rows(&self) -> Option<Vec<LearnedPhraseRow>> {
        self.database.wait_for_queued_writes();
        self.database.read(|connection| {
            let mut statement = connection.prepare(&format!(
                "SELECT hanzi, roman, learn_count FROM {TABLE_NAME}\nORDER BY learn_count DESC, updated_at DESC, id;"
            ))?;
            let rows = statement.query_map([], decode_row)?;
            rows.collect()
        })
    }

    /// Rebuilds every learned phrase's search keys when the stored database predates
    /// the Hakfa initial derivation (v1 → v2: Hakfa `ch-` preserved in Tl family)
    /// or the engine has not taken the file over yet, then records the version and takeover.
    pub fn rederive_search_keys_if_needed(&self) -> Result<(), UserDataDatabaseError> {
        let stored: Option<Vec<(i64, String)>> = self.database.perform::<_, UserDataDatabaseError>(|connection| {
            let version = user_version(connection)?;
            if version >= SCHEMA_VERSION && is_taken_over(connection)? {
                return Ok(None);
            }
            let mut statement = connection.prepare(&format!(
                "SELECT id, roman FROM {TABLE_NAME};"
            ))?;
            let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            let mut list = Vec::new();
            for r in rows {
                list.push(r?);
            }
            Ok(Some(list))
        })?;

        let Some(stored) = stored else {
            return Ok(());
        };

        let mut derived_by_roman: HashMap<String, Vec<CustomSearchKey>> = HashMap::new();
        for (_, roman) in &stored {
            if derived_by_roman.contains_key(roman) {
                continue;
            }
            if let Some(keys) = (self.derive_search_keys)(roman).filter(|k| !k.is_empty()) {
                derived_by_roman.insert(roman.clone(), keys);
            }
        }

        self.database.perform::<_, UserDataDatabaseError>(move |connection| {
            immediate_transaction(connection, |connection| {
                for (id, roman) in &stored {
                    if let Some(keys) = derived_by_roman.get(roman) {
                        delete_search_keys(connection, *id)?;
                        write_search_keys(connection, *id, keys)?;
                    }
                }
                let version = user_version(connection)?;
                connection.pragma_update(None, "user_version", version.max(SCHEMA_VERSION))?;
                mark_taken_over(connection)?;
                Ok(())
            })
        })
    }

    /// What launch or open runs once the file is open: the takeover's key re-derivation.
    pub fn finish_takeover(&self) {
        if let Err(error) = self.rederive_search_keys_if_needed() {
            log::error!("learned_phrases.rederive_failed error={error}");
        }
    }

    /// Forgets everything — the learning-data wipe, beside the frequency and
    /// association stores'. One transaction: the keys and the rows go
    /// together.
    pub fn delete_all(&self) -> Result<(), UserDataDatabaseError> {
        self.database.perform(|connection| {
            immediate_transaction::<_, rusqlite::Error>(connection, |connection| {
                sync_ledger::block_rows(connection, TABLE_NAME, ["hanzi", "roman"], "1", [])?;
                connection.execute(&format!("DELETE FROM {SEARCH_KEY_TABLE_NAME};"), [])?;
                connection.execute(&format!("DELETE FROM {TABLE_NAME};"), [])?;
                Ok(())
            })?;
            connection.execute("VACUUM;", []).ok();
            Ok(())
        })
    }
}

fn decode_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LearnedPhraseRow> {
    Ok(LearnedPhraseRow {
        hanji: row.get(0)?,
        canonical_tl: row.get(1)?,
        learn_count: row.get(2)?,
    })
}

/// The keys of one phrase, deleted with it.
fn delete_search_keys(connection: &Connection, phrase_id: i64) -> rusqlite::Result<()> {
    connection.execute(
        &format!("DELETE FROM {SEARCH_KEY_TABLE_NAME} WHERE phrase_id = ?;"),
        params![phrase_id],
    )?;
    Ok(())
}

/// The full cross-mode key bundle for a fresh row. `OR IGNORE` on the
/// side-key unique index absorbs a repeated key.
fn write_search_keys(
    connection: &Connection,
    phrase_id: i64,
    search_keys: &[CustomSearchKey],
) -> rusqlite::Result<()> {
    let mut statement = connection.prepare(&format!(
        "INSERT OR IGNORE INTO {SEARCH_KEY_TABLE_NAME} (phrase_id, family, form, key)\nVALUES (?, ?, ?, ?);"
    ))?;
    for key in search_keys {
        statement.execute(params![phrase_id, key.family, key.form, key.key])?;
    }
    Ok(())
}

/// Drops rows past `cap`, never `kept_id` (the row just inserted survives
/// whatever its timestamp ties with). A cheap `COUNT(*)` first — under the
/// cap the ordered walk never runs. Keys first, so the subquery still
/// resolves against the intact main table; `OFFSET cap - 1` selects exactly
/// the rows past the cap once the kept row is set aside. Ported
/// from iOS `evictPastCap` / Android `PAST_CAP_SQL`.
fn evict_past_cap(connection: &Connection, cap: usize, kept_id: i64) -> rusqlite::Result<()> {
    let count: i64 =
        connection.query_row(&format!("SELECT COUNT(*) FROM {TABLE_NAME};"), [], |row| {
            row.get(0)
        })?;
    if count <= cap as i64 {
        return Ok(());
    }
    let past_cap = format!(
        "SELECT id FROM {TABLE_NAME}\nWHERE id <> ?\nORDER BY learn_count DESC, updated_at DESC, id\nLIMIT -1 OFFSET ?"
    );
    let offset = cap.saturating_sub(1) as i64;
    connection.execute(
        &format!("DELETE FROM {SEARCH_KEY_TABLE_NAME} WHERE phrase_id IN ({past_cap});"),
        params![kept_id, offset],
    )?;
    connection.execute(
        &format!("DELETE FROM {TABLE_NAME} WHERE id IN ({past_cap});"),
        params![kept_id, offset],
    )?;
    Ok(())
}

/// The current shape, created directly. Ported from
/// iOS `LearnedPhraseSchema` / Android `LearnedPhraseService` DDL.
fn apply_schema(connection: &Connection) -> rusqlite::Result<()> {
    connection.execute_batch(&format!(
        "CREATE TABLE IF NOT EXISTS {TABLE_NAME} (\n    id INTEGER PRIMARY KEY,\n    roman TEXT NOT NULL,\n    hanzi TEXT NOT NULL,\n    learn_count INTEGER NOT NULL DEFAULT 1,\n    updated_at TEXT NOT NULL,\n    UNIQUE(hanzi, roman)\n);\nCREATE TABLE IF NOT EXISTS {SEARCH_KEY_TABLE_NAME} (\n    phrase_id INTEGER NOT NULL,\n    family TEXT NOT NULL,\n    form TEXT NOT NULL,\n    key TEXT NOT NULL\n);\nCREATE INDEX IF NOT EXISTS idx_lsk_lookup ON {SEARCH_KEY_TABLE_NAME}(family, form, key);\nCREATE UNIQUE INDEX IF NOT EXISTS idx_lsk_phrase ON {SEARCH_KEY_TABLE_NAME}(phrase_id, family, form, key);\nCREATE INDEX IF NOT EXISTS idx_learned_rank ON {TABLE_NAME}(learn_count, updated_at);"
    ))?;
    connection.execute_batch(&sync_ledger::ddl())?;
    Ok(())
}
