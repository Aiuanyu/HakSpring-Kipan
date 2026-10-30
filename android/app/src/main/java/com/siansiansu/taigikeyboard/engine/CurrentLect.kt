package com.siansiansu.taigikeyboard.engine

/**
 * Hakfa fork: the lect the keyboard is typing, as the engine's lect code (1 四縣, 6 南四縣, 2 海陸 — the
 * `Lect::code` values in `desktop/crates/taigi-desktop-core/src/settings/choices.rs`). The merged
 * dictionary holds every lect; 0 would mix their readings into one list.
 *
 * The system input-method menu is the source: each `<subtype>` in `res/xml/method.xml` carries
 * `lect=<id>` in its `imeSubtypeExtraValue`, and `TaigiKeyboard` hands the active one to [setFromExtraValue].
 * Read wherever the bridge builds an engine config ([appConfig], [dictionaryTogglesProto]).
 */
object CurrentLect {
    const val XIIEN = 1
    const val NAMXIIAN = 6
    const val HOILIUG = 2

    @Volatile
    var code: Int = XIIEN
        private set

    /** The lect code a subtype's extra value names (`lect=xiien`, ...); 四縣 for anything else. */
    fun codeFor(extraValue: String?): Int =
        when (extraValue?.split(',')?.map { it.trim() }?.firstOrNull { it.startsWith("lect=") }?.removePrefix("lect=")) {
            "namxiian" -> NAMXIIAN
            "hoiliug" -> HOILIUG
            else -> XIIEN
        }

    /** Sets the lect from a subtype's extra value; `true` when it changed. */
    fun setFromExtraValue(extraValue: String?): Boolean {
        val next = codeFor(extraValue)
        if (next == code) return false
        code = next
        return true
    }
}
