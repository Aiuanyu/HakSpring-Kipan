package com.siansiansu.taigikeyboard.engine

import android.content.Context
import org.json.JSONObject

/**
 * Hakfa fork: metadata about the bundled dictionary assets from `assets/data_version.json`.
 */
data class LexiconVersionInfo(
    val version: String,
    val builtAt: String,
    val recordCount: Int,
) {
    fun displaySummary(): String =
        if (recordCount > 0) {
            val formatted = java.text.NumberFormat.getIntegerInstance().format(recordCount)
            "$version（$formatted 條詞）"
        } else {
            version
        }
}

object LexiconVersion {
    private const val ASSET_NAME = "data_version.json"

    @Volatile
    private var cached: LexiconVersionInfo? = null

    fun get(context: Context): LexiconVersionInfo {
        cached?.let { return it }
        val info = try {
            context.assets.open(ASSET_NAME).bufferedReader().use { reader ->
                val json = JSONObject(reader.readText())
                LexiconVersionInfo(
                    version = json.optString("hakspring_data_version", "unknown"),
                    builtAt = json.optString("built_at", ""),
                    recordCount = json.optInt("record_count", 0),
                )
            }
        } catch (_: Exception) {
            LexiconVersionInfo("unknown", "", 0)
        }
        cached = info
        return info
    }
}
