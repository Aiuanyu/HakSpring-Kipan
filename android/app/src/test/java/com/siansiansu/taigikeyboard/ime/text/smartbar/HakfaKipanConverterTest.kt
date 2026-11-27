package com.siansiansu.taigikeyboard.ime.text.smartbar

import com.siansiansu.taigikeyboard.engine.proto.ConvertibleTargetResult
import com.siansiansu.taigikeyboard.engine.proto.KipanPair
import com.siansiansu.taigikeyboard.engine.proto.KipanTwoLinesResult
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

/**
 * Contract tests for [HakfaKipanConverter] (mapping, empty-input short-circuits,
 * and fallback behavior when the Rust JNI bridge is unavailable on host JVM).
 *
 * Full conversion algorithm tests live in Rust `engine/two-line` (`cargo test -p two-line`)
 * and macOS `KipanConverterTests.swift`.
 */
class HakfaKipanConverterTest {

    @After
    fun restoreBackend() {
        // Ensure default backend isn't left mutated across tests.
        HakfaKipanConverter.backend = object : HakfaKipanConverter.Backend {
            override fun convertPunctToRomance(text: String) = text
            override fun convertPunctToHan(text: String) = text
            override fun convertKipanTwoLines(
                text: String,
                autoSpaceRomance: Boolean,
                forceOrder: String,
            ): KipanTwoLinesResult? = null
            override fun extractConvertibleTarget(textBeforeCursor: String): ConvertibleTargetResult? = null
        }
    }

    @Test
    fun mapsProtoResultToConversionResult() {
        HakfaKipanConverter.backend = object : HakfaKipanConverter.Backend {
            override fun convertPunctToRomance(text: String) = "rom:$text"
            override fun convertPunctToHan(text: String) = "han:$text"
            override fun convertKipanTwoLines(
                text: String,
                autoSpaceRomance: Boolean,
                forceOrder: String,
            ): KipanTwoLinesResult = KipanTwoLinesResult
                .newBuilder()
                .setTopText("Python 係𠊎用 Rust")
                .setBottomText("Python he Ngǎi iung Rust")
                .setCombinedText("Python 係𠊎用 Rust\nPython he Ngǎi iung Rust")
                .addTopLines("Python 係𠊎用 Rust")
                .addBottomLines("Python he Ngǎi iung Rust")
                .addPairs(KipanPair.newBuilder().setHan("係").setRom("he"))
                .setDetectedOrder(if (forceOrder == "auto") "han-top" else forceOrder)
                .build()

            override fun extractConvertibleTarget(textBeforeCursor: String): ConvertibleTargetResult =
                ConvertibleTargetResult
                    .newBuilder()
                    .setFound(true)
                    .setDeleteBeforeUtf16Len(textBeforeCursor.length)
                    .setText(textBeforeCursor)
                    .build()
        }

        assertEquals("rom:。", HakfaKipanConverter.convertPunctToRomance("。"))
        assertEquals("han:.", HakfaKipanConverter.convertPunctToHan("."))

        val res = HakfaKipanConverter.convertKipanText("Python 係 (he) 𠊎 (Ngǎi)")
        assertEquals("han-top", res.detectedOrder)
        assertEquals("Python 係𠊎用 Rust", res.topText)
        assertEquals("Python he Ngǎi iung Rust", res.bottomText)
        assertEquals(listOf(HakfaKipanConverter.Pair("係", "he")), res.pairs)

        val target = HakfaKipanConverter.extractConvertibleTarget("𠊎 (Ngǎi)")
        assertEquals("𠊎 (Ngǎi)", target?.text)
        assertEquals("𠊎 (Ngǎi)".length, target?.deleteBeforeLength)
    }

    @Test
    fun emptyAndNotFoundShortCircuit() {
        HakfaKipanConverter.backend = object : HakfaKipanConverter.Backend {
            override fun convertPunctToRomance(text: String) = error("should not be called")
            override fun convertPunctToHan(text: String) = error("should not be called")
            override fun convertKipanTwoLines(
                text: String,
                autoSpaceRomance: Boolean,
                forceOrder: String,
            ): KipanTwoLinesResult? = error("should not be called")
            override fun extractConvertibleTarget(textBeforeCursor: String): ConvertibleTargetResult =
                ConvertibleTargetResult.newBuilder().setFound(false).build()
        }

        assertEquals("", HakfaKipanConverter.convertPunctToRomance(""))
        assertEquals("", HakfaKipanConverter.convertPunctToHan(""))
        val emptyRes = HakfaKipanConverter.convertKipanText("")
        assertEquals("", emptyRes.topText)
        assertTrue(emptyRes.pairs.isEmpty())
        assertNull(HakfaKipanConverter.extractConvertibleTarget(""))
        assertNull(HakfaKipanConverter.extractConvertibleTarget("純粹漢字無標註"))
    }
}
