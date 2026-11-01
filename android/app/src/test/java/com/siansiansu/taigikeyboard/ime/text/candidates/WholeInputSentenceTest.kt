package com.siansiansu.taigikeyboard.ime.text.candidates

import com.siansiansu.taigikeyboard.ime.dictionary.TaigiWord
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/** Hakfa fork: punctuation commits the sentence the engine composed for all of the pending input. */
class WholeInputSentenceTest {
    private fun cell(
        id: Int,
        hanji: String?,
        roman: String,
        consumedBytes: Int,
        coverageKind: Int,
        syllables: Int = 1,
    ) = TaigiWord(
        id = id,
        roman = roman,
        hanji = hanji,
        additionalInfo =
            mapOf(
                TaigiWord.MetadataKeys.IS_CONTINUOUS to "true",
                TaigiWord.MetadataKeys.CONSUMED_BYTES to consumedBytes.toString(),
                TaigiWord.MetadataKeys.SYLLABLE_COUNT to syllables.toString(),
                TaigiWord.MetadataKeys.DISPLAY_TEXT to (hanji ?: roman),
                TaigiWord.MetadataKeys.CANONICAL_TL to roman,
                TaigiWord.MetadataKeys.COVERAGE_KIND to coverageKind.toString(),
            ),
    )

    @Test
    fun theFirstWholeReadingThatEndsWhereThePendingInputEndsIsThePick() {
        val words =
            listOf(
                cell(1, null, "ngaigiongzu", 11, 0), // the literal romanization: no Hanji
                cell(2, "𠊎薑著", "ngǎi gióng zu", 11, 0, syllables = 3),
                cell(3, "𠊎", "ngǎi", 4, 0),
            )
        val pick = wholeInputSentencePick(words, pendingBytes = 11)
        assertNotNull(pick)
        assertEquals("𠊎薑著", pick!!.canonicalText)
        assertEquals(11, pick.consumedBytes)
    }

    @Test
    fun aCompletionOfAHalfTypedSyllableOrAnAbbreviationIsNotASentence() {
        val prefix = listOf(cell(1, "大", "tai", 1, 1))
        val abbreviation = listOf(cell(1, "青竹絲", "qiáng-zùg-xí", 3, 2, syllables = 3))
        assertNull(wholeInputSentencePick(prefix, pendingBytes = 1))
        assertNull(wholeInputSentencePick(abbreviation, pendingBytes = 3))
    }

    @Test
    fun aCandidateThatStopsShortOfThePendingInputIsNotASentence() {
        val words = listOf(cell(1, "𠊎", "ngǎi", 4, 0))
        assertNull(wholeInputSentencePick(words, pendingBytes = 11))
        assertNull(wholeInputSentencePick(words, pendingBytes = 0))
    }

    @Test
    fun onlyMarksThatEndAPhraseSendTheSentence() {
        for (mark in listOf(",", ".", "?", "!", "，", "。", "？", "；", "「")) assertTrue(mark, isSentencePunctuation(mark))
        for (other in listOf("", "a", "5", " ", "字", "\n", "\u200B", "\u200D", "👩‍💻")) assertFalse(other, isSentencePunctuation(other))
    }
}
