package com.siansiansu.taigikeyboard.ime.dictionary

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Hakfa fork: custom words joined across lects, and the writes a save makes. */
class CustomWordTest {
    private val lects = listOf(1, 2, 6)

    private fun row(
        id: String,
        hanji: String,
        roman: String,
        lect: Int,
    ) = CustomDictionaryWord(id = id, roman = roman, hanji = hanji, lect = lect)

    @Test
    fun rowsOfOneHanjiJoinIntoOneWordWithAReadingPerLect() {
        val words =
            groupCustomWords(
                listOf(row("a", "𠊎", "ngai", 1), row("b", "𠊎", "ngoi", 2), row("c", "瑋", "vi", 1)),
                lects,
            )
        assertEquals(listOf("𠊎", "瑋"), words.map { it.hanji })
        assertEquals("ngai", words[0].roman(1))
        assertEquals("ngoi", words[0].roman(2))
        assertEquals("", words[0].roman(6))
        assertEquals(listOf(1), words[1].readings.keys.toList())
    }

    @Test
    fun aRomanOnlyEntryStandsAloneAndALectZeroRowFillsEveryEmptySlot() {
        val words = groupCustomWords(listOf(row("a", "", "xx", 1), row("b", "", "yy", 1), row("g", "字", "zi", 0)), lects)
        assertEquals(3, words.size)
        val generic = words.last()
        assertEquals(listOf(1, 2, 6), generic.readings.keys.toList())
        assertEquals(1, generic.rows.size)
    }

    @Test
    fun savingKeepsTheIdOfAReadingThatStaysAndDeletesTheOnesThatGoAway() {
        val original = groupCustomWords(listOf(row("a", "𠊎", "ngai", 1), row("b", "𠊎", "ngoi", 2)), lects).single()
        val plan = planWordSave(original, "𠊎", mapOf(1 to "ngai", 2 to "", 6 to "ngai"), lects) { "new" }
        assertEquals(listOf("a", "new"), plan.saves.map { it.id })
        assertEquals(listOf(1, 6), plan.saves.map { it.lect })
        assertEquals(listOf("b"), plan.deleteIds)
    }

    @Test
    fun aNewWordWritesOneRowPerFilledLectAndDeletesNothing() {
        var next = 0
        val plan = planWordSave(null, " 水 ", mapOf(1 to " sui ", 2 to "   "), lects) { "id${next++}" }
        assertEquals(listOf(row("id0", "水", "sui", 1)), plan.saves)
        assertTrue(plan.deleteIds.isEmpty())
    }

    @Test
    fun aLectZeroRowIsReplacedByTheLectSpecificOnesNotReused() {
        val original = groupCustomWords(listOf(row("g", "字", "zi", 0)), lects).single()
        val plan = planWordSave(original, "字", mapOf(1 to "zi", 2 to "zi"), lects) { "fresh" }
        assertTrue("g" !in plan.saves.map { it.id })
        assertEquals(listOf("g"), plan.deleteIds)
    }
}
