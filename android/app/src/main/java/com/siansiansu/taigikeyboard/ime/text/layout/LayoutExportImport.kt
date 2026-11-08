package com.siansiansu.taigikeyboard.ime.text.layout

import org.json.JSONArray
import org.json.JSONObject
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.UUID

/**
 * Handles exporting and importing custom keyboard layouts in JSON format,
 * supporting both single-layout files and multi-layout backup packages.
 */
object LayoutExportImport {
    const val TYPE = "kipan_layout"
    const val BACKUP_TYPE = "kipan_layouts_backup"
    const val CURRENT_SCHEMA_VERSION = 1

    /** Encodes a single [CustomLayout] into a portable JSON string. */
    fun exportLayout(layout: CustomLayout): String {
        val root =
            JSONObject().apply {
                put("type", TYPE)
                put("schemaVersion", CURRENT_SCHEMA_VERSION)
                put("exportedAt", System.currentTimeMillis())
                put("layout", layout.toJson())
            }
        return root.toString(2)
    }

    /** Encodes all [layouts] into a multi-layout backup JSON string. */
    fun exportAllLayouts(layouts: List<CustomLayout>): String {
        val root =
            JSONObject().apply {
                put("type", BACKUP_TYPE)
                put("schemaVersion", CURRENT_SCHEMA_VERSION)
                put("exportedAt", System.currentTimeMillis())
                put("layouts", JSONArray().apply { layouts.forEach { put(it.toJson()) } })
            }
        return root.toString(2)
    }

    /** Proposes a safe, human-readable file name for exporting [layoutName]. */
    fun suggestFileName(layoutName: String): String {
        val safe =
            layoutName
                .trim()
                .replace(Regex("[^a-zA-Z0-9\\u4e00-\\u9fa5_-]"), "_")
                .ifBlank { "layout" }
        return "kipan-layout-$safe.json"
    }

    /** Proposes a timestamped file name for a full layouts backup. */
    fun suggestBackupFileName(): String {
        val date = SimpleDateFormat("yyyyMMdd", Locale.US).format(Date())
        return "kipan-layouts-backup-$date.json"
    }

    /**
     * Decodes and validates one or more custom layouts from [jsonString] against [tables].
     *
     * Accepts:
     * - Multi-layout package: `{"type": "kipan_layouts_backup", "layouts": [...]}`
     * - Bare JSON array of layouts: `[{...}, {...}]`
     * - Single-layout package: `{"type": "kipan_layout", "layout": {...}}`
     * - Bare CustomLayout object: `{"id": "...", "base": "...", "script": "...", "keys": [...]}`
     */
    fun importLayouts(
        jsonString: String,
        tables: CustomLayoutTables,
        existingLayouts: List<CustomLayout> = emptyList(),
    ): Result<List<CustomLayout>> =
        runCatching {
            if (jsonString.isBlank()) throw IllegalArgumentException("Empty content")
            val trimmed = jsonString.trim()

            val rawLayoutObjects: List<JSONObject> =
                if (trimmed.startsWith("[")) {
                    val array = JSONArray(trimmed)
                    (0 until array.length()).mapNotNull { array.optJSONObject(it) }
                } else {
                    val root = JSONObject(trimmed)
                    when {
                        root.has("layouts") -> {
                            val array = root.getJSONArray("layouts")
                            (0 until array.length()).mapNotNull { array.optJSONObject(it) }
                        }
                        root.has("layout") -> listOf(root.getJSONObject("layout"))
                        root.has("base") && root.has("script") -> listOf(root)
                        else -> throw IllegalArgumentException("Unrecognized layout JSON structure")
                    }
                }

            if (rawLayoutObjects.isEmpty()) {
                throw IllegalArgumentException("No valid layout entries found in JSON")
            }

            val currentNames = existingLayouts.map { it.name }.toMutableList()
            rawLayoutObjects.mapNotNull { obj ->
                val decoded = CustomLayout.fromJson(obj, tables) ?: return@mapNotNull null
                if (!tables.isValid(decoded)) return@mapNotNull null

                val newId = UUID.randomUUID().toString()
                val originalName = decoded.name.ifBlank { "Imported Layout" }
                val finalName = resolveUniqueName(originalName, currentNames)
                currentNames.add(finalName)

                decoded.copy(
                    id = newId,
                    name = finalName,
                )
            }.also {
                if (it.isEmpty()) throw IllegalArgumentException("Failed to decode any valid custom layouts")
            }
        }

    /** Convenience method to import a single custom layout. */
    fun importLayout(
        jsonString: String,
        tables: CustomLayoutTables,
        existingLayouts: List<CustomLayout> = emptyList(),
    ): Result<CustomLayout> =
        importLayouts(jsonString, tables, existingLayouts).mapCatching { list ->
            list.firstOrNull() ?: throw IllegalArgumentException("No valid layout found")
        }

    private fun resolveUniqueName(
        baseName: String,
        existingNames: List<String>,
    ): String {
        if (baseName !in existingNames) return baseName
        var counter = 1
        while ("$baseName ($counter)" in existingNames) {
            counter++
        }
        return "$baseName ($counter)"
    }
}
