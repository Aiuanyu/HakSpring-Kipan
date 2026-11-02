package com.siansiansu.taigikeyboard.ime.settings

import org.junit.Assert.assertEquals
import org.junit.Test

/** Hakfa fork: the four Annotate in Brackets templates — what they write, how a cell previews them, how they persist. */
class BracketTemplateTest {
    @Test
    fun wrap_writesTheLeadThenTheAnnotationInEachTemplate() {
        assertEquals("臺灣 (tǒi-vǎn)", BracketTemplate.ROUND.wrap("臺灣", "tǒi-vǎn"))
        assertEquals("臺灣 [tǒi-vǎn]", BracketTemplate.SQUARE.wrap("臺灣", "tǒi-vǎn"))
        assertEquals("臺灣 {tǒi-vǎn}", BracketTemplate.CURLY.wrap("臺灣", "tǒi-vǎn"))
        assertEquals("<ruby>臺灣<rt>tǒi-vǎn</rt></ruby>", BracketTemplate.RUBY.wrap("臺灣", "tǒi-vǎn"))
    }

    @Test
    fun shortWrap_isWrapWithRubyTagsTrimmed() {
        assertEquals("漢 [羅]", BracketTemplate.SQUARE.shortWrap("漢", "羅"))
        assertEquals("羅 <rt>漢", BracketTemplate.RUBY.shortWrap("羅", "漢"))
    }

    @Test
    fun preview_showsTheBracketsButRubyOnlyAMarkAhead() {
        assertEquals("(x)", BracketTemplate.ROUND.preview("x"))
        assertEquals("[x]", BracketTemplate.SQUARE.preview("x"))
        assertEquals("{x}", BracketTemplate.CURLY.preview("x"))
        assertEquals("ᴿᵀ x", BracketTemplate.RUBY.preview("x"))
    }

    @Test
    fun storage_roundTripsAndAnUnknownValueFallsBackToSquare() {
        BracketTemplate.entries.forEach { assertEquals(it, BracketTemplate.fromStorage(it.storageValue)) }
        assertEquals(BracketTemplate.SQUARE, BracketTemplate.fromStorage(null))
        assertEquals(BracketTemplate.SQUARE, BracketTemplate.fromStorage("bogus"))
    }

    @Test
    fun annotation_picksTheTemplateOfTheLead() {
        val annotation = BracketAnnotation(hanjiFirst = BracketTemplate.SQUARE, romanFirst = BracketTemplate.CURLY)
        assertEquals(BracketTemplate.SQUARE, annotation.templateFor(isHanjiFirst = true))
        assertEquals(BracketTemplate.CURLY, annotation.templateFor(isHanjiFirst = false))
    }
}
