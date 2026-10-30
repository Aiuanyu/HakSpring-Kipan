//! Hakfa fork (各腔互學): who owns a learned row, kept beside the shared learning tables so their
//! shape — the one iOS and Android also read — does not change.
//!
//! - `lect_sync_copy`: pairs the lect sync wrote. A copy is not the user's evidence, so the sync
//!   never offers it to the other lects again. A real pick, a count edit or a backup import of the
//!   same pair makes the row the user's own: [`claim`] drops it from this ledger.
//! - `lect_sync_block`: pairs the user deleted (one row, or everything). The sync does not copy a
//!   blocked pair back. Learning the pair again [`claim`]s it too.
//!
//! Both are keyed by the same `(text, TL)` pair as the table they sit next to. A pre-ledger copy is
//! indistinguishable from the user's own pick, so the ledger starts empty and old rows count as
//! the user's.

use rusqlite::{params, Connection};
use std::collections::HashSet;

pub(crate) const COPY_TABLE: &str = "lect_sync_copy";
pub(crate) const BLOCK_TABLE: &str = "lect_sync_block";

/// The two tables, safe to run on every open.
pub(crate) fn ddl() -> String {
    format!(
        "CREATE TABLE IF NOT EXISTS {COPY_TABLE} (\n    word TEXT NOT NULL,\n    tl TEXT NOT NULL,\n    PRIMARY KEY (word, tl)\n);\nCREATE TABLE IF NOT EXISTS {BLOCK_TABLE} (\n    word TEXT NOT NULL,\n    tl TEXT NOT NULL,\n    PRIMARY KEY (word, tl)\n);"
    )
}

/// The user learned or edited this pair: it is theirs, and no longer blocked.
pub(crate) fn claim(connection: &Connection, word: &str, tl: &str) -> rusqlite::Result<()> {
    connection.execute(
        &format!("DELETE FROM {COPY_TABLE} WHERE word = ? AND tl = ?;"),
        params![word, tl],
    )?;
    connection.execute(
        &format!("DELETE FROM {BLOCK_TABLE} WHERE word = ? AND tl = ?;"),
        params![word, tl],
    )?;
    Ok(())
}

/// The sync wrote this pair.
pub(crate) fn mark_copied(connection: &Connection, word: &str, tl: &str) -> rusqlite::Result<()> {
    connection.execute(
        &format!("INSERT OR IGNORE INTO {COPY_TABLE} (word, tl) VALUES (?, ?);"),
        params![word, tl],
    )?;
    Ok(())
}

/// The user is deleting the rows of `table` that `condition` selects (`1` for all of them): block
/// their pairs. Run before the `DELETE`, inside its transaction. A copy entry of a deleted pair is
/// left in place: it names no row, and learning the pair again [`claim`]s it.
pub(crate) fn block_rows(
    connection: &Connection,
    table: &str,
    columns: [&str; 2],
    condition: &str,
    values: impl rusqlite::Params,
) -> rusqlite::Result<()> {
    let [text, tl] = columns;
    connection.execute(
        &format!("INSERT OR IGNORE INTO {BLOCK_TABLE} (word, tl) SELECT {text}, {tl} FROM {table} WHERE {condition};"),
        values,
    )?;
    Ok(())
}

/// Every pair of one ledger table.
pub(crate) fn read_pairs(
    connection: &Connection,
    table: &str,
) -> rusqlite::Result<HashSet<(String, String)>> {
    let mut statement = connection.prepare(&format!("SELECT word, tl FROM {table};"))?;
    let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    rows.collect()
}
