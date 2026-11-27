package com.siansiansu.taigikeyboard.ime.text.smartbar

import com.siansiansu.taigikeyboard.engine.RustEngineBridge
import com.siansiansu.taigikeyboard.engine.convertKipanTwoLines
import com.siansiansu.taigikeyboard.engine.convertPunctToHan
import com.siansiansu.taigikeyboard.engine.convertPunctToRomance
import com.siansiansu.taigikeyboard.engine.extractConvertibleTarget
import com.siansiansu.taigikeyboard.engine.proto.ConvertibleTargetResult
import com.siansiansu.taigikeyboard.engine.proto.KipanTwoLinesResult

/**
 * Hakfa fork: Converts inline dual-script Kipan annotations
 * (e.g. `字詞 (拼音)` or `拼音 (字詞)` or `<ruby>字<rt>im</rt></ruby>`)
 * into clean two-line parallel text (Hanji line + Romanization line).
 *
 * Delegates to the shared Rust `two-line` engine via [RustEngineBridge].
 */
object HakfaKipanConverter {

    data class Pair(
        val han: String,
        val rom: String,
    )

    data class ConversionResult(
        val topText: String,
        val bottomText: String,
        val combinedText: String,
        val topLines: List<String>,
        val bottomLines: List<String>,
        val pairs: List<Pair>,
        val detectedOrder: String,
    )

    data class TargetSlice(
        val deleteBeforeLength: Int,
        val text: String,
    )

    internal interface Backend {
        fun convertPunctToRomance(text: String): String
        fun convertPunctToHan(text: String): String
        fun convertKipanTwoLines(
            text: String,
            autoSpaceRomance: Boolean,
            forceOrder: String,
        ): KipanTwoLinesResult?
        fun extractConvertibleTarget(textBeforeCursor: String): ConvertibleTargetResult?
    }

    private object RustBackend : Backend {
        override fun convertPunctToRomance(text: String): String =
            RustEngineBridge.convertPunctToRomance(text)

        override fun convertPunctToHan(text: String): String =
            RustEngineBridge.convertPunctToHan(text)

        override fun convertKipanTwoLines(
            text: String,
            autoSpaceRomance: Boolean,
            forceOrder: String,
        ): KipanTwoLinesResult? =
            RustEngineBridge.convertKipanTwoLines(text, autoSpaceRomance, forceOrder)

        override fun extractConvertibleTarget(textBeforeCursor: String): ConvertibleTargetResult? =
            RustEngineBridge.extractConvertibleTarget(textBeforeCursor)
    }

    @Volatile
    internal var backend: Backend = RustBackend

    fun convertPunctToRomance(p: String): String {
        if (p.isEmpty()) return ""
        return backend.convertPunctToRomance(p)
    }

    fun convertPunctToHan(p: String): String {
        if (p.isEmpty()) return ""
        return backend.convertPunctToHan(p)
    }

    fun convertKipanText(
        text: String,
        forceOrder: String = "auto",
    ): ConversionResult {
        if (text.isEmpty()) {
            return ConversionResult(
                topText = "",
                bottomText = "",
                combinedText = "",
                topLines = emptyList(),
                bottomLines = emptyList(),
                pairs = emptyList(),
                detectedOrder = "han-top",
            )
        }
        val res = backend.convertKipanTwoLines(
            text = text,
            autoSpaceRomance = false,
            forceOrder = forceOrder,
        ) ?: return ConversionResult(
            topText = text,
            bottomText = text,
            combinedText = "$text\n$text",
            topLines = text.split("\n"),
            bottomLines = text.split("\n"),
            pairs = emptyList(),
            detectedOrder = "han-top",
        )
        return ConversionResult(
            topText = res.topText,
            bottomText = res.bottomText,
            combinedText = res.combinedText,
            topLines = res.topLinesList,
            bottomLines = res.bottomLinesList,
            pairs = res.pairsList.map { Pair(han = it.han, rom = it.rom) },
            detectedOrder = res.detectedOrder,
        )
    }

    fun extractConvertibleTarget(textBeforeCursor: String): TargetSlice? {
        if (textBeforeCursor.isEmpty()) return null
        val res = backend.extractConvertibleTarget(textBeforeCursor) ?: return null
        if (!res.found) return null
        return TargetSlice(
            deleteBeforeLength = res.deleteBeforeUtf16Len,
            text = res.text,
        )
    }
}
