package com.siansiansu.taigikeyboard.ime.text.smartbar

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class HakfaKipanConverterTest {

    @Test
    fun standardExample_hanTop_convertsDunhaoToComma_preservesHyphenAndSpace() {
        val input =
            "𠊎 (Ngǎi) 無 (mǒ) 幾愛 (gì oi) 客委會 (Hàg-Ví-Fi) 認 (ngin) 證辭彙 (ziin cǐifi)裡肚 (dídù)有兜仔 (iúdéu'è)个 (ge)例 (li) 句 (gi)會 (voi)講 (gòng)體 (tì) 罰 (fad)、權 (kiǎn) 威 (ví)教育 (gau'iùg)个 (ge)感覺 (gàmgòg)"

        val res = HakfaKipanConverter.convertKipanText(input)

        assertEquals("han-top", res.detectedOrder)
        assertEquals(
            "𠊎無幾愛客委會認證辭彙裡肚有兜仔个例句會講體罰、權威教育个感覺",
            res.topText,
        )
        assertEquals(
            "Ngǎi mǒ gì oi Hàg-Ví-Fi ngin ziin cǐifi dídù iúdéu'è ge li gi voi gòng tì fad, kiǎn ví gau'iùg ge gàmgòg",
            res.bottomText,
        )
    }

    @Test
    fun manualForceOrder_romTop_swapsRows() {
        val input = "𠊎 (Ngǎi) 無 (mǒ) 幾愛 (gì oi) 客委會 (Hàg-Ví-Fi)"

        val resRomTop = HakfaKipanConverter.convertKipanText(input, forceOrder = "rom-top")
        assertEquals("rom-top", resRomTop.detectedOrder)
        assertEquals("Ngǎi mǒ gì oi Hàg-Ví-Fi", resRomTop.topText)
        assertEquals("𠊎無幾愛客委會", resRomTop.bottomText)

        val resHanTop = HakfaKipanConverter.convertKipanText(input, forceOrder = "han-top")
        assertEquals("han-top", resHanTop.detectedOrder)
        assertEquals("𠊎無幾愛客委會", resHanTop.topText)
        assertEquals("Ngǎi mǒ gì oi Hàg-Ví-Fi", resHanTop.bottomText)
    }

    @Test
    fun punctuationBidirectionalConversion() {
        val hanPunct = "「食飽、無，恁好。敢係？當好！」（四縣）【美濃】｛註｝"
        assertEquals(
            "\"食飽,無,恁好.敢係?當好!\"(四縣)[美濃]{註}",
            HakfaKipanConverter.convertPunctToRomance(hanPunct),
        )

        val romPunct = "Ngǎi, ngi. he? ho! (Sixian) [Meinung] {Note}"
        assertEquals(
            "Ngǎi， ngi。 he？ ho！ （Sixian） ［Meinung］ ｛Note｝",
            HakfaKipanConverter.convertPunctToHan(romPunct),
        )

        val inputWithPunct = "試驗 (Cii-ngiàm)：好 (hò)！【認證】 (Ngin-ziin)……"
        val res = HakfaKipanConverter.convertKipanText(inputWithPunct)
        assertEquals("試驗：好！【認證】……", res.topText)
        assertEquals("Cii-ngiàm: hò! [Ngin-ziin]...", res.bottomText)
    }

    @Test
    fun reverseCase_romanFirst_automaticallyRomTop() {
        val input = "Ngǎi (𠊎) mǒ (無) gì oi (幾愛) Hàg-Ví-Fi (客委會)！"
        val res = HakfaKipanConverter.convertKipanText(input)

        assertEquals("rom-top", res.detectedOrder)
        assertEquals("Ngǎi mǒ gì oi Hàg-Ví-Fi!", res.topText)
        assertEquals("𠊎無幾愛客委會！", res.bottomText)
    }

    @Test
    fun fullWidthAndSquareAndCurlyBracketsCompatibility() {
        val fullWidth = HakfaKipanConverter.convertKipanText("客源翠（Hàg-Ngiǎn-Cui） 企盤（Kì-Pǎn）")
        assertEquals("客源翠企盤", fullWidth.topText)
        assertEquals("Hàg-Ngiǎn-Cui Kì-Pǎn", fullWidth.bottomText)

        val square = HakfaKipanConverter.convertKipanText("無成功 [mǒ sǐin'gúng] 無反應 [mǒ fàn'in]")
        assertEquals("han-top", square.detectedOrder)
        assertEquals("無成功無反應\nmǒ sǐin'gúng mǒ fàn'in", square.combinedText)

        val curly = HakfaKipanConverter.convertKipanText("試驗 {Cii-ngiàm} 好 {hò}")
        assertEquals("試驗好", curly.topText)
        assertEquals("Cii-ngiàm hò", curly.bottomText)
    }

    @Test
    fun rubyAndRtTagsCompatibility() {
        val input = "<ruby>𠊎<rt>Ngǎi</rt></ruby><ruby>識<rt>sii</rt></ruby><ruby>講<rt>gòng</rt></ruby><ruby>客<rt>Hàg</rt></ruby>。"
        val res = HakfaKipanConverter.convertKipanText(input)

        assertEquals("𠊎識講客。", res.topText)
        assertEquals("Ngǎi sii gòng Hàg.", res.bottomText)
        assertEquals("𠊎識講客。\nNgǎi sii gòng Hàg.", res.combinedText)
    }

    @Test
    fun neutralTextPreservedInBothLines() {
        val input = "這幾下日 (ià giá ngìd) 做到恁仔 (zo do àn'é)，有想愛 (iú xiòng'oi) 推到 (túi do) v0.0.5 吔 (ě)"
        val res = HakfaKipanConverter.convertKipanText(input)

        assertEquals("這幾下日做到恁仔，有想愛推到 v0.0.5 吔", res.topText)
        assertEquals("ià giá ngìd zo do àn'é, iú xiòng'oi túi do v0.0.5 ě", res.bottomText)
    }

    @Test
    fun multiParagraphs_preservesLineBreaks() {
        val input = "𠊎 (Ngǎi) 識 (sii) 講 (gòng) 客 (Hàg)。\n汝 (Ngi) 呢 (nē)？"
        val res = HakfaKipanConverter.convertKipanText(input)

        assertEquals("𠊎識講客。\n汝呢？", res.topText)
        assertEquals("Ngǎi sii gòng Hàg.\nNgi nē?", res.bottomText)
    }

    @Test
    fun plainTextWithoutAnnotations_hasNoPairs() {
        val res = HakfaKipanConverter.convertKipanText("無括號个純文字")
        assertTrue(res.pairs.isEmpty())
    }

    @Test
    fun extractConvertibleTarget_prefersCurrentParagraph() {
        val before = "第一行無標註\n𠊎 (Ngǎi) 講 (gòng) 客 (Hàg)"
        val target = HakfaKipanConverter.extractConvertibleTarget(before)
        assertEquals("𠊎 (Ngǎi) 講 (gòng) 客 (Hàg)", target?.text)
        assertEquals("𠊎 (Ngǎi) 講 (gòng) 客 (Hàg)".length, target?.deleteBeforeLength)
    }

    @Test
    fun extractConvertibleTarget_fallsBackToFullBeforeCursorWhenTrailingNewline() {
        val before = "𠊎 (Ngǎi) 講 (gòng) 客 (Hàg)\n"
        val target = HakfaKipanConverter.extractConvertibleTarget(before)
        assertEquals(before, target?.text)
        assertEquals(before.length, target?.deleteBeforeLength)
    }

    @Test
    fun extractConvertibleTarget_returnsNullWhenNothingConvertible() {
        assertNull(HakfaKipanConverter.extractConvertibleTarget(""))
        assertNull(HakfaKipanConverter.extractConvertibleTarget("純粹漢字無標註"))
    }
}
