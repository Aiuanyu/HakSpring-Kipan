package com.siansiansu.taigikeyboard.ui.tabs.settings

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

/** Hakfa fork: device summary stays separate from full diagnostic text with crash logs. */
class DiagnosticServiceTest {
    private fun sample(recentCrashes: String = "") =
        DiagnosticInfo(
            appVersion = "0.0.3",
            buildNumber = 42,
            lexiconVersion = "2026111022",
            activeLectName = "四縣",
            osVersion = "14",
            deviceModel = "Google Pixel",
            recentCrashes = recentCrashes,
        )

    @Test
    fun formattedDeviceInfo_omitsRecentCrashesEvenWhenPresent() {
        val info = sample(recentCrashes = "java.lang.IllegalStateException: boom")
        val deviceOnly = info.formattedDeviceInfo()

        assertTrue(deviceOnly.contains("App: v0.0.3 (42)"))
        assertTrue(deviceOnly.contains("Device: Google Pixel"))
        assertFalse(deviceOnly.contains("Recent crashes:"))
        assertFalse(deviceOnly.contains("boom"))
    }

    @Test
    fun formatted_matchesDeviceInfoWhenNoCrashes() {
        val info = sample(recentCrashes = "")
        assertEquals(info.formattedDeviceInfo(), info.formatted())
    }

    @Test
    fun formatted_appendsRecentCrashesWhenPresent() {
        val info = sample(recentCrashes = "java.lang.IllegalStateException: boom")
        val full = info.formatted()

        assertTrue(full.startsWith(info.formattedDeviceInfo()))
        assertTrue(full.contains("\n\nRecent crashes:\njava.lang.IllegalStateException: boom"))
    }
}
