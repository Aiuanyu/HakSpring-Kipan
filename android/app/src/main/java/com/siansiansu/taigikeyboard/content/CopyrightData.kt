package com.siansiansu.taigikeyboard.content

import com.siansiansu.taigikeyboard.i18n.StringResolver
import com.siansiansu.taigikeyboard.i18n.generated.StringKey

// Copyright data for fonts, dictionaries, and software displayed in the Home tab

private const val SIL_OFL_LICENSE_URL = "https://openfontlicense.org/"

// License identifiers are standardized short codes — language-invariant, never translated (kept as
// constants rather than i18n keys per the multi-language plan; SPDX-aligned).
private const val SIL_OPEN_FONT_LICENSE = "SIL Open Font License"
private const val SIL_OPEN_FONT_LICENSE_11 = "SIL Open Font License 1.1"
private const val CC_BY_ND_3_TW_LICENSE = "CC BY-ND 3.0 TW"
private const val OGDL_TAIWAN_10_LICENSE = "OGDL-Taiwan-1.0"
private const val APACHE_2_LICENSE = "Apache License 2.0"

data class CopyrightPage(
    val id: Int,
    val title: String,
    val description: String,
    val license: String,
    val buttons: List<CopyrightButton>,
)

data class CopyrightButton(
    val text: String,
    val url: String,
)

object CopyrightDataSource {
    // Resolver-driven so the i18n strings re-resolve under the active display language.
    fun copyrightPages(resolver: StringResolver): List<CopyrightPage> =
        listOf(
            // Fonts
            // 0. 丸仔烏體（Maruko Gothic）
            CopyrightPage(
                id = 0,
                title = resolver.resolve(StringKey.COMMON_FONT_MARUKO_GOTHIC),
                description = "© The Maruko Gothic Project Authors (syimyuz) / Carlito © Google Inc.",
                license = SIL_OPEN_FONT_LICENSE_11,
                buttons =
                    listOf(
                        CopyrightButton(
                            text = resolver.resolve(StringKey.HOME_VIEW_LICENSE),
                            url = SIL_OFL_LICENSE_URL,
                        ),
                        CopyrightButton(
                            text = resolver.resolve(StringKey.COMMON_VIEW_WEBSITE),
                            url = "https://github.com/syimyuz/maruko-gothic",
                        ),
                    ),
            ),
            // 1. 芫荽
            CopyrightPage(
                id = 1,
                title = resolver.resolve(StringKey.COMMON_FONT_IANSUI),
                description = resolver.resolve(StringKey.HOME_BUT_TAIWAN_COPYRIGHT),
                license = SIL_OPEN_FONT_LICENSE_11,
                buttons =
                    listOf(
                        CopyrightButton(
                            text = resolver.resolve(StringKey.HOME_VIEW_LICENSE),
                            url = SIL_OFL_LICENSE_URL,
                        ),
                        CopyrightButton(
                            text = resolver.resolve(StringKey.COMMON_VIEW_WEBSITE),
                            url = "https://github.com/ButTaiwan/iansui",
                        ),
                    ),
            ),
            // 2. jf open 粉圓 2.1
            CopyrightPage(
                id = 2,
                title = resolver.resolve(StringKey.COMMON_FONT_OPEN_HUNINN),
                description = resolver.resolve(StringKey.HOME_OPEN_FONT_COPYRIGHT),
                license = SIL_OPEN_FONT_LICENSE,
                buttons =
                    listOf(
                        CopyrightButton(
                            text = resolver.resolve(StringKey.HOME_VIEW_LICENSE),
                            url = SIL_OFL_LICENSE_URL,
                        ),
                        CopyrightButton(
                            text = resolver.resolve(StringKey.COMMON_VIEW_WEBSITE),
                            url = "https://justfont.com/huninn/",
                        ),
                    ),
            ),
            // 3. 源樣明體
            CopyrightPage(
                id = 3,
                title = resolver.resolve(StringKey.COMMON_FONT_GEN_YO_MIN),
                description = resolver.resolve(StringKey.HOME_BUT_TAIWAN_COPYRIGHT),
                license = SIL_OPEN_FONT_LICENSE_11,
                buttons =
                    listOf(
                        CopyrightButton(
                            text = resolver.resolve(StringKey.HOME_VIEW_LICENSE),
                            url = SIL_OFL_LICENSE_URL,
                        ),
                        CopyrightButton(
                            text = resolver.resolve(StringKey.COMMON_VIEW_WEBSITE),
                            url = "https://github.com/ButTaiwan/genyo-font",
                        ),
                    ),
            ),
            // 4. 源樣烏體
            CopyrightPage(
                id = 4,
                title = resolver.resolve(StringKey.COMMON_FONT_GEN_YO_GOTHIC),
                description = resolver.resolve(StringKey.HOME_BUT_TAIWAN_COPYRIGHT),
                license = SIL_OPEN_FONT_LICENSE_11,
                buttons =
                    listOf(
                        CopyrightButton(
                            text = resolver.resolve(StringKey.HOME_VIEW_LICENSE),
                            url = SIL_OFL_LICENSE_URL,
                        ),
                        CopyrightButton(
                            text = resolver.resolve(StringKey.COMMON_VIEW_WEBSITE),
                            url = "https://github.com/ButTaiwan/genyog-font",
                        ),
                    ),
            ),
            // Dictionaries
            // 10. 教育部臺灣客語辭典
            CopyrightPage(
                id = 10,
                title = resolver.resolve(StringKey.COMMON_MOE_DICT),
                description = resolver.resolve(StringKey.HOME_MOE_COPYRIGHT),
                license = CC_BY_ND_3_TW_LICENSE,
                buttons =
                    listOf(
                        CopyrightButton(
                            text = resolver.resolve(StringKey.HOME_VIEW_LICENSE),
                            url = "https://creativecommons.org/licenses/by-nd/3.0/tw/",
                        ),
                        CopyrightButton(
                            text = resolver.resolve(StringKey.COMMON_VIEW_WEBSITE),
                            url = "https://hakkadict.moe.edu.tw/",
                        ),
                    ),
            ),
            // 11. 客語能力認證詞彙
            CopyrightPage(
                id = 11,
                title = resolver.resolve(StringKey.COMMON_NEWWORD_DICT),
                description = resolver.resolve(StringKey.HOME_NEWWORD_COPYRIGHT),
                license = OGDL_TAIWAN_10_LICENSE,
                buttons =
                    listOf(
                        CopyrightButton(
                            text = resolver.resolve(StringKey.HOME_VIEW_LICENSE),
                            url = "https://spdx.org/licenses/OGDL-Taiwan-1.0.html",
                        ),
                        CopyrightButton(
                            text = resolver.resolve(StringKey.COMMON_VIEW_WEBSITE),
                            url = "https://elearning.hakka.gov.tw/hakka/download-files",
                        ),
                    ),
            ),
            // Upstream Software
            // 20. TaigiKeyboard
            CopyrightPage(
                id = 20,
                title = "TaigiKeyboard",
                description = "© 2025-2026 蘇民弦（Soo Bîn-hiân）",
                license = APACHE_2_LICENSE,
                buttons =
                    listOf(
                        CopyrightButton(
                            text = resolver.resolve(StringKey.HOME_VIEW_LICENSE),
                            url = "https://www.apache.org/licenses/LICENSE-2.0",
                        ),
                        CopyrightButton(
                            text = resolver.resolve(StringKey.COMMON_VIEW_WEBSITE),
                            url = "https://github.com/taigikeyboard/taigikeyboard",
                        ),
                    ),
            ),
        )
}
