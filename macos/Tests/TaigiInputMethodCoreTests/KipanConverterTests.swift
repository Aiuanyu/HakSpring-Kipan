// KipanConverterTests.swift
// 測試客源翠企盤打字並列格式轉換器（KipanConverter Swift 移植版）

import XCTest
@testable import TaigiInputMethodCore

final class KipanConverterTests: XCTestCase {

    func testStandardExample_hanTop_convertsDunhaoToComma_preservesHyphenAndSpace() {
        let input = "𠊎 (Ngǎi) 無 (mǒ) 幾愛 (gì oi) 客委會 (Hàg-Ví-Fi) 認 (ngin) 證辭彙 (ziin cǐifi)裡肚 (dídù)有兜仔 (iúdéu'è)个 (ge)例 (li) 句 (gi)會 (voi)講 (gòng)體 (tì) 罰 (fad)、權 (kiǎn) 威 (ví)教育 (gau'iùg)个 (ge)感覺 (gàmgòg)"

        let res = KipanConverter.convertKipanText(input)

        XCTAssertEqual(res.detectedOrder, "han-top", "應自動偵測諸夏字在頂高")
        XCTAssertEqual(
            res.topText,
            "𠊎無幾愛客委會認證辭彙裡肚有兜仔个例句會講體罰、權威教育个感覺",
            "頂層諸夏字行應自動去除雜亂空格"
        )
        XCTAssertEqual(
            res.bottomText,
            "Ngǎi mǒ gì oi Hàg-Ví-Fi ngin ziin cǐifi dídù iúdéu'è ge li gi voi gòng tì fad, kiǎn ví gau'iùg ge gàmgòg",
            "底層羅馬字行頓號應轉換為逗號「, 」，並保留 hyphen、內部空格與連寫"
        )
    }

    func testManualForceOrder_romTop_swapsRows() {
        let input = "𠊎 (Ngǎi) 無 (mǒ) 幾愛 (gì oi) 客委會 (Hàg-Ví-Fi)"

        let resRomTop = KipanConverter.convertKipanText(input, forceOrder: "rom-top")
        XCTAssertEqual(resRomTop.detectedOrder, "rom-top")
        XCTAssertEqual(resRomTop.topText, "Ngǎi mǒ gì oi Hàg-Ví-Fi", "頂層必須為真正的羅馬字行")
        XCTAssertEqual(resRomTop.bottomText, "𠊎無幾愛客委會", "底層必須為真正的諸夏字行")

        let resHanTop = KipanConverter.convertKipanText(input, forceOrder: "han-top")
        XCTAssertEqual(resHanTop.detectedOrder, "han-top")
        XCTAssertEqual(resHanTop.topText, "𠊎無幾愛客委會")
        XCTAssertEqual(resHanTop.bottomText, "Ngǎi mǒ gì oi Hàg-Ví-Fi")
    }

    func testPunctuationBidirectionalConversion() {
        // 1. 中文全形轉羅馬字半形
        let hanPunct = "「食飽、無，恁好。敢係？當好！」（四縣）【美濃】｛註｝"
        XCTAssertEqual(
            KipanConverter.convertPunctToRomance(hanPunct),
            "\"食飽,無,恁好.敢係?當好!\"(四縣)[美濃]{註}"
        )

        // 2. 羅馬字半形轉中文全形
        let romPunct = "Ngǎi, ngi. he? ho! (Sixian) [Meinung] {Note}"
        XCTAssertEqual(
            KipanConverter.convertPunctToHan(romPunct),
            "Ngǎi， ngi。 he？ ho！ （Sixian） ［Meinung］ ｛Note｝"
        )

        // 3. 完整雙向語句轉換驗證（含括號）
        let inputWithPunct = "試驗 (Cii-ngiàm)：好 (hò)！【認證】 (Ngin-ziin)……"
        let res1 = KipanConverter.convertKipanText(inputWithPunct)
        XCTAssertEqual(res1.topText, "試驗：好！【認證】……")
        XCTAssertEqual(res1.bottomText, "Cii-ngiàm: hò! [Ngin-ziin]...")
    }

    func testReverseCase_romanFirst_automaticallyRomTop() {
        let input = "Ngǎi (𠊎) mǒ (無) gì oi (幾愛) Hàg-Ví-Fi (客委會)！"

        let res = KipanConverter.convertKipanText(input)
        XCTAssertEqual(res.detectedOrder, "rom-top", "應自動偵測羅馬字在頂高")
        XCTAssertEqual(res.topText, "Ngǎi mǒ gì oi Hàg-Ví-Fi!", "頂層羅馬字標點應為半形「!」")
        XCTAssertEqual(res.bottomText, "𠊎無幾愛客委會！", "底層諸夏字標點應為全形「！」")
    }

    func testFullWidthBracketsCompatibility() {
        let input = "客源翠（Hàg-Ngiǎn-Cui） 企盤（Kì-Pǎn）"
        let res = KipanConverter.convertKipanText(input)

        XCTAssertEqual(res.topText, "客源翠企盤")
        XCTAssertEqual(res.bottomText, "Hàg-Ngiǎn-Cui Kì-Pǎn")
    }

    func testSquareBracketsCompatibility() {
        let input = "無成功 [mǒ sǐin'gúng] 無反應 [mǒ fàn'in]"
        let res = KipanConverter.convertKipanText(input)

        XCTAssertEqual(res.detectedOrder, "han-top")
        XCTAssertEqual(res.topText, "無成功無反應")
        XCTAssertEqual(res.bottomText, "mǒ sǐin'gúng mǒ fàn'in")
        XCTAssertEqual(res.combinedText, "無成功無反應\nmǒ sǐin'gúng mǒ fàn'in", "雙行之間應僅換一行")
    }

    func testCurlyBracketsCompatibility() {
        let input = "試驗 {Cii-ngiàm} 好 {hò}"
        let res = KipanConverter.convertKipanText(input)

        XCTAssertEqual(res.topText, "試驗好")
        XCTAssertEqual(res.bottomText, "Cii-ngiàm hò")
    }

    func testRubyAndRtTagsCompatibility() {
        let input = "<ruby>𠊎<rt>Ngǎi</rt></ruby><ruby>識<rt>sii</rt></ruby><ruby>講<rt>gòng</rt></ruby><ruby>客<rt>Hàg</rt></ruby>。"
        let res = KipanConverter.convertKipanText(input)

        XCTAssertEqual(res.topText, "𠊎識講客。")
        XCTAssertEqual(res.bottomText, "Ngǎi sii gòng Hàg.")
        XCTAssertEqual(res.combinedText, "𠊎識講客。\nNgǎi sii gòng Hàg.")
    }

    func testNeutralTextPreservedInBothLines() {
        let input = "這幾下日 (ià giá ngìd) 做到恁仔 (zo do àn'é)，有想愛 (iú xiòng'oi) 推到 (túi do) v0.0.5 吔 (ě)"
        let res = KipanConverter.convertKipanText(input)

        XCTAssertEqual(res.topText, "這幾下日做到恁仔，有想愛推到 v0.0.5 吔", "諸夏字行應保留 v0.0.5")
        XCTAssertEqual(res.bottomText, "ià giá ngìd zo do àn'é, iú xiòng'oi túi do v0.0.5 ě", "羅馬字行也應保留 v0.0.5")
    }

    func testMultiParagraphs_preservesLineBreaks() {
        let input = "𠊎 (Ngǎi) 識 (sii) 講 (gòng) 客 (Hàg)。\n汝 (Ngi) 呢 (nē)？"
        let res = KipanConverter.convertKipanText(input)

        let expectedTop = "𠊎識講客。\n汝呢？"
        let expectedBottom = "Ngǎi sii gòng Hàg.\nNgi nē?"

        XCTAssertEqual(res.topText, expectedTop, "諸夏字行使用全形中文標點「。」與「？」")
        XCTAssertEqual(res.bottomText, expectedBottom, "羅馬字行使用半形西洋標點「.」與「?」")
    }

    func testBoundaryDefense_emptyOrPlainString() {
        XCTAssertEqual(KipanConverter.convertKipanText("").topText, "")

        let plain = "純文字無括號"
        let resPlain = KipanConverter.convertKipanText(plain)
        XCTAssertEqual(resPlain.topText, plain)
        XCTAssertEqual(resPlain.bottomText, plain)
    }
}
