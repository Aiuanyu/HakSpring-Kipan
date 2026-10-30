package com.siansiansu.taigikeyboard.engine

import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

/** Hakfa fork: the lect every request is made in, and how a keyboard subtype names it. */
class HakfaLectsTest {
    @After
    fun resetActiveLect() {
        ActiveLect.current = ActiveLect.XIIEN
    }

    @Test
    fun aSubtypeNamesItsLectInItsExtraValue() {
        assertEquals(1, HakfaLects.fromSubtypeExtra("lect=1"))
        assertEquals(2, HakfaLects.fromSubtypeExtra("lect=2"))
        assertEquals(6, HakfaLects.fromSubtypeExtra("lect=6"))
        assertEquals(2, HakfaLects.fromSubtypeExtra("other=x, lect=2"))
    }

    @Test
    fun anUnknownOrMissingLectIsNotAnAnswer() {
        assertNull(HakfaLects.fromSubtypeExtra(null))
        assertNull(HakfaLects.fromSubtypeExtra(""))
        assertNull(HakfaLects.fromSubtypeExtra("lect=3"))
        assertNull(HakfaLects.fromSubtypeExtra("lect=abc"))
    }

    @Test
    fun theMenuListsTheThreeBuiltLectsInTheProjectOrder() {
        // 四南海大平安: 四縣, 南四縣, 海陸 first.
        assertEquals(listOf(1, 6, 2), HakfaLects.all.map { it.code })
    }

    @Test
    fun theWireConfigCarriesTheActiveLectUnlessTheCallerNamesOne() {
        assertEquals(1, appConfig("tl").lect)
        ActiveLect.current = ActiveLect.HOILIUG
        assertEquals(2, appConfig("tl").lect)
        assertEquals(6, appConfig("tl", lect = ActiveLect.NAMXIIAN).lect)
    }
}
