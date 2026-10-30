// Hakfa fork: the custom dictionary's rows grouped by Hanji, one reading per lect (Mac's four-column sheet).

package com.siansiansu.taigikeyboard.ime.dictionary

import java.util.UUID

/**
 * One custom word as the screens show it: a Hanji and the reading it has in each lect. The engine
 * stores one row per (word, lect); this joins them back. A word with no Hanji (a roman-only entry)
 * stands alone under its row id.
 */
data class CustomWord(
    val key: String,
    val hanji: String,
    /** Lect code → the stored row holding that lect's reading. */
    val readings: Map<Int, CustomDictionaryWord>,
) {
    fun roman(lect: Int): String = readings[lect]?.roman.orEmpty()

    /** Every stored row of the word, once each (a lect-0 row can stand for several lects). */
    val rows: List<CustomDictionaryWord> get() = readings.values.distinctBy { it.id }
}

/**
 * Groups [rows] into words, keeping the order each word first appears in. A row's lect decides its
 * slot; a lect-0 row (any lect) fills every empty slot and a row of a lect [lects] does not name
 * fills the first slot, as Mac does.
 */
fun groupCustomWords(
    rows: List<CustomDictionaryWord>,
    lects: List<Int>,
): List<CustomWord> {
    val slots = LinkedHashMap<String, MutableMap<Int, CustomDictionaryWord>>()
    val hanjiOf = HashMap<String, String>()
    for (row in rows) {
        val key = row.hanji.ifBlank { row.id }
        hanjiOf.putIfAbsent(key, row.hanji)
        val slot = slots.getOrPut(key) { LinkedHashMap() }
        when {
            row.lect in lects -> slot[row.lect] = row
            row.lect == 0 -> lects.forEach { slot.putIfAbsent(it, row) }
            else -> lects.firstOrNull()?.let { slot.putIfAbsent(it, row) }
        }
    }
    return slots.map { (key, slot) -> CustomWord(key, hanjiOf.getValue(key), slot) }
}

/** What saving a word changes: rows to write, and the old rows to remove. */
data class WordSavePlan(
    val saves: List<CustomDictionaryWord>,
    val deleteIds: List<String>,
)

/**
 * The writes that turn [original] (or nothing, for a new word) into [hanji] with the readings in
 * [romans] (lect code → text): a non-blank reading is saved under its lect (keeping the row id it
 * already had), a blank one saves nothing, and every old row not kept is deleted — so a cleared
 * reading removes that lect's row, and a lect-0 row is replaced by the lect-specific ones.
 */
fun planWordSave(
    original: CustomWord?,
    hanji: String,
    romans: Map<Int, String>,
    lects: List<Int>,
    newId: () -> String = { UUID.randomUUID().toString() },
): WordSavePlan {
    val saves =
        lects.mapNotNull { lect ->
            val roman = romans[lect]?.trim().orEmpty()
            if (roman.isEmpty()) return@mapNotNull null
            val id = original
                ?.readings
                ?.get(lect)
                ?.takeIf { it.lect == lect }
                ?.id ?: newId()
            CustomDictionaryWord(id = id, roman = roman, hanji = hanji.trim(), lect = lect)
        }
    val kept = saves.map { it.id }.toSet()
    val deletes = original
        ?.rows
        ?.map { it.id }
        ?.filter { it !in kept }
        .orEmpty()
    return WordSavePlan(saves, deletes)
}
