// Hakfa fork: the lect (腔) every engine request is made in.

package com.siansiansu.taigikeyboard.engine

/**
 * The lect the keyboard types in, read by every request builder (`appConfig`,
 * `dictionaryTogglesProto`). A plain process-wide value: the keyboard service and the settings
 * screens share one process, and `PrefHelper.lect` is the persisted copy that keeps this one
 * current. Top-level and Kotlin-only so JVM tests can pin the wire (like `appConfig`).
 *
 * Lect codes are the dictionary's (`docs/hakfa/lect.md`): 1 四縣, 6 南四縣, 2 海陸, 3 大埔, 4 饒平, 5 詔安.
 */
object ActiveLect {
    const val XIIEN = 1
    const val NAMXIIAN = 6
    const val HOILIUG = 2
    const val TAIBU = 3
    const val NGIAUPIN = 4
    const val ZHIOON = 5

    @Volatile
    var current: Int = XIIEN
}

/**
 * The lects Android offers, with the names the other platforms show. The order is the project's
 * everywhere (a menu, a column, a file): 四縣, 南四縣, 海陸, then 大埔, 饒平, 詔安 (「四南海大平安」).
 */
object HakfaLects {
    data class Lect(
        val code: Int,
        val name: String,
    )

    val all: List<Lect> =
        listOf(
            Lect(ActiveLect.XIIEN, "四縣 (Xi'ien)"),
            Lect(ActiveLect.NAMXIIAN, "南四縣 (Nǎm Xi'ian)"),
            Lect(ActiveLect.HOILIUG, "海陸 (Hóiliug)"),
            Lect(ActiveLect.TAIBU, "大埔 (Tàibú)"),
            Lect(ActiveLect.NGIAUPIN, "饒平 (Ngiaupin)"),
            Lect(ActiveLect.ZHIOON, "詔安 (Zhiô'ǒn)"),
        )

    fun isKnown(code: Int): Boolean = all.any { it.code == code }

    fun nameOf(code: Int): String = all.firstOrNull { it.code == code }?.name ?: all.first().name

    /** The code a subtype's `imeSubtypeExtraValue` (`lect=2`) names; `null` for anything else. */
    fun fromSubtypeExtra(extra: String?): Int? =
        extra
            ?.split(',')
            ?.map { it.trim() }
            ?.firstOrNull { it.startsWith("lect=") }
            ?.removePrefix("lect=")
            ?.toIntOrNull()
            ?.takeIf(::isKnown)
}
