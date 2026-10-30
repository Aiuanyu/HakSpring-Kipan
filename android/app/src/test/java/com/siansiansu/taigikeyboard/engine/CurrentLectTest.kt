package com.siansiansu.taigikeyboard.engine

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** Hakfa fork: the input-method subtype's extra value picks the engine lect (`res/xml/method.xml`). */
class CurrentLectTest {
    // trace: the three `lect=` values in method.xml -> Lect::code 1 / 6 / 2 (choices.rs)
    @Test
    fun codeForReadsTheLectFromTheExtraValue() {
        assertEquals(CurrentLect.XIIEN, CurrentLect.codeFor("lect=xiien"))
        assertEquals(CurrentLect.NAMXIIAN, CurrentLect.codeFor("lect=namxiian"))
        assertEquals(CurrentLect.HOILIUG, CurrentLect.codeFor("lect=hoiliug"))
    }

    @Test
    fun codeForFindsTheLectAmongOtherExtraValues() {
        assertEquals(CurrentLect.HOILIUG, CurrentLect.codeFor("other=1, lect=hoiliug"))
    }

    // trace: no subtype / an unknown one falls to 四縣, never 0 (0 mixes every lect's readings)
    @Test
    fun codeForFallsToXiienForAnythingElse() {
        assertEquals(CurrentLect.XIIEN, CurrentLect.codeFor(null))
        assertEquals(CurrentLect.XIIEN, CurrentLect.codeFor(""))
        assertEquals(CurrentLect.XIIEN, CurrentLect.codeFor("lect=unknown"))
    }

    @Test
    fun setFromExtraValueReportsAChangeOnce() {
        try {
            CurrentLect.setFromExtraValue("lect=xiien")
            assertTrue(CurrentLect.setFromExtraValue("lect=hoiliug"))
            assertEquals(CurrentLect.HOILIUG, CurrentLect.code)
            assertFalse(CurrentLect.setFromExtraValue("lect=hoiliug"))
        } finally {
            CurrentLect.setFromExtraValue("lect=xiien")
        }
    }
}
