package com.siansiansu.taigikeyboard.ime.core

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

/** Hakfa fork: the crash file keeps the newest entries whole and stays within its size cap. */
class CrashLogTest {
    @Test
    fun entry_carriesTheTimeThreadAndStack() {
        val text = CrashLog.entry("main", IllegalStateException("boom"), nowMs = 0L)

        assertTrue(text, text.startsWith("1970-01-01 08:00:00 (UTC+8) thread=main\n"))
        assertTrue(text, text.contains("java.lang.IllegalStateException: boom"))
    }

    @Test
    fun appendEntry_joinsEntriesOldestFirst() {
        assertEquals("a", CrashLog.appendEntry("", "a"))
        assertEquals("a\n=====\nb", CrashLog.appendEntry("a", "b"))
    }

    @Test
    fun appendEntry_dropsTheOldestWholeEntriesPastTheCap() {
        var log = ""
        repeat(10) { n -> log = CrashLog.appendEntry(log, "entry-$n " + "x".repeat(5_000)) }

        assertTrue(log.length <= 24_000)
        assertTrue("the newest survives", log.contains("entry-9 "))
        assertTrue("the oldest is gone", !log.contains("entry-0 "))
        assertTrue("the oldest kept entry starts whole", log.startsWith("entry-"))
    }
}
