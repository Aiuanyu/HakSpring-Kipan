package com.siansiansu.taigikeyboard.ime.theme

import org.json.JSONArray
import org.json.JSONObject
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.UUID

/**
 * Handles exporting and importing user-created themes in JSON format,
 * supporting both single-theme files and multi-theme backup packages.
 */
object ThemeExportImport {
    const val TYPE = "kipan_theme"
    const val BACKUP_TYPE = "kipan_themes_backup"
    const val CURRENT_SCHEMA_VERSION = 1

    /** Encodes a single [UserTheme] into a portable JSON string. */
    fun exportTheme(theme: UserTheme): String {
        val root =
            JSONObject().apply {
                put("type", TYPE)
                put("schemaVersion", CURRENT_SCHEMA_VERSION)
                put("exportedAt", System.currentTimeMillis())
                put("theme", theme.toJson())
            }
        return root.toString(2)
    }

    /** Encodes all [themes] into a multi-theme backup JSON string. */
    fun exportAllThemes(themes: List<UserTheme>): String {
        val root =
            JSONObject().apply {
                put("type", BACKUP_TYPE)
                put("schemaVersion", CURRENT_SCHEMA_VERSION)
                put("exportedAt", System.currentTimeMillis())
                put("themes", JSONArray().apply { themes.forEach { put(it.toJson()) } })
            }
        return root.toString(2)
    }

    /** Proposes a safe, human-readable file name for exporting [themeName]. */
    fun suggestFileName(themeName: String): String {
        val safe =
            themeName
                .trim()
                .replace(Regex("[^a-zA-Z0-9\\u4e00-\\u9fa5_-]"), "_")
                .ifBlank { "theme" }
        return "kipan-theme-$safe.json"
    }

    /** Proposes a timestamped file name for a full themes backup. */
    fun suggestBackupFileName(): String {
        val date = SimpleDateFormat("yyyyMMdd", Locale.US).format(Date())
        return "kipan-themes-backup-$date.json"
    }

    /**
     * Decodes and validates one or more themes from [jsonString].
     *
     * Accepts:
     * - Multi-theme package: `{"type": "kipan_themes_backup", "themes": [...]}`
     * - Bare JSON array of themes: `[{...}, {...}]`
     * - Single-theme package: `{"type": "kipan_theme", "theme": {...}}`
     * - Bare UserTheme object: `{"id": "...", "appearance": {...}}`
     */
    fun importThemes(
        jsonString: String,
        existingThemes: List<UserTheme> = emptyList(),
    ): Result<List<UserTheme>> =
        runCatching {
            if (jsonString.isBlank()) throw IllegalArgumentException("Empty content")
            val trimmed = jsonString.trim()

            val rawThemeObjects: List<JSONObject> =
                if (trimmed.startsWith("[")) {
                    val array = JSONArray(trimmed)
                    (0 until array.length()).mapNotNull { array.optJSONObject(it) }
                } else {
                    val root = JSONObject(trimmed)
                    when {
                        root.has("themes") -> {
                            val array = root.getJSONArray("themes")
                            (0 until array.length()).mapNotNull { array.optJSONObject(it) }
                        }
                        root.has("theme") -> listOf(root.getJSONObject("theme"))
                        root.has("appearance") -> listOf(root)
                        else -> throw IllegalArgumentException("Unrecognized theme JSON structure")
                    }
                }

            if (rawThemeObjects.isEmpty()) {
                throw IllegalArgumentException("No valid theme entries found in JSON")
            }

            val currentNames = existingThemes.map { it.name }.toMutableList()
            rawThemeObjects.mapNotNull { obj ->
                val decoded = UserTheme.fromJson(obj) ?: return@mapNotNull null
                val newId = UUID.randomUUID().toString()
                val originalName = decoded.name.ifBlank { "Imported Theme" }
                val finalName = resolveUniqueName(originalName, currentNames)
                currentNames.add(finalName)

                val now = System.currentTimeMillis()
                UserTheme(
                    id = newId,
                    name = finalName,
                    appearance = decoded.appearance,
                    createdAt = now,
                    updatedAt = now,
                )
            }.also {
                if (it.isEmpty()) throw IllegalArgumentException("Failed to decode any themes")
            }
        }

    /** Convenience method to import a single theme. */
    fun importTheme(
        jsonString: String,
        existingThemes: List<UserTheme> = emptyList(),
    ): Result<UserTheme> =
        importThemes(jsonString, existingThemes).mapCatching { list ->
            list.firstOrNull() ?: throw IllegalArgumentException("No theme found")
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
