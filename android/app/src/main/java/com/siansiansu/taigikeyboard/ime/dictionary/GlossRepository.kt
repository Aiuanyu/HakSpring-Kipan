package com.siansiansu.taigikeyboard.ime.dictionary

import android.content.Context
import android.database.sqlite.SQLiteDatabase
import java.io.File
import java.io.FileOutputStream

data class GlossEntry(
    val id: Long,
    val word: String,
    val lectId: String,
    val lectName: String,
    val reading: String,
    val gloss: String,
    val synonyms: List<String>,
    val source: String,
    val dialectTags: String,
    val isSynonymHit: Boolean = false,
    val matchedKeyword: String? = null,
)

/**
 * Hakfa fork: Repository for querying assets/dictionaries/gloss.db (SQLite).
 * Supports exact & prefix matching for Hanji headwords and Chinese/Hakka synonym keywords.
 */
class GlossRepository(private val context: Context) {

    private val dbFile = File(context.filesDir, "gloss.db")

    private val database: SQLiteDatabase? by lazy {
        ensureDatabaseCopied()
        try {
            if (dbFile.exists()) {
                SQLiteDatabase.openDatabase(dbFile.path, null, SQLiteDatabase.OPEN_READONLY)
            } else {
                null
            }
        } catch (e: Exception) {
            null
        }
    }

    private fun ensureDatabaseCopied() {
        if (!dbFile.exists()) {
            try {
                context.assets.open("gloss.db").use { input ->
                    FileOutputStream(dbFile).use { output ->
                        input.copyTo(output)
                    }
                }
            } catch (ignored: Exception) {
            }
        }
    }

    /**
     * Look up headwords and synonyms for a given query text.
     * Orders results by "四南海大平安" or places `lectFilter` first if specified.
     */
    fun search(query: String, lectFilter: String? = null): List<GlossEntry> {
        val trimmed = query.trim()
        if (trimmed.isEmpty()) return emptyList()

        val db = database ?: return emptyList()
        val results = mutableListOf<GlossEntry>()
        val seenKeys = mutableSetOf<String>() // (word, lectId) dedup

        val lectOrderClause = if (lectFilter != null) {
            """
            CASE e.lect_id
                WHEN '$lectFilter' THEN 0
                WHEN 'xiien' THEN 1
                WHEN 'namxiian' THEN 2
                WHEN 'hoiliug' THEN 3
                WHEN 'taibu' THEN 4
                WHEN 'ngiaupin' THEN 5
                WHEN 'zhioon' THEN 6
                ELSE 7
            END
            """.trimIndent()
        } else {
            """
            CASE e.lect_id
                WHEN 'xiien' THEN 1
                WHEN 'namxiian' THEN 2
                WHEN 'hoiliug' THEN 3
                WHEN 'taibu' THEN 4
                WHEN 'ngiaupin' THEN 5
                WHEN 'zhioon' THEN 6
                ELSE 7
            END
            """.trimIndent()
        }

        // 1. 同形詞精確比對 (Headword exact match)
        val wordSql = """
            SELECT e.id, e.word, e.lect_id, e.lect_name, e.reading, e.gloss, e.synonyms, e.source, e.dialect_tags
            FROM entries e
            WHERE e.word = ?
            ORDER BY $lectOrderClause
        """.trimIndent()

        db.rawQuery(wordSql, arrayOf(trimmed)).use { cursor ->
            while (cursor.moveToNext()) {
                val id = cursor.getLong(0)
                val word = cursor.getString(1)
                val lectId = cursor.getString(2)
                val lectName = cursor.getString(3)
                val reading = cursor.getString(4)
                val gloss = cursor.getString(5)
                val synsStr = cursor.getString(6)
                val source = cursor.getString(7)
                val tags = cursor.getString(8)
                val syns = if (synsStr.isNotEmpty()) synsStr.split(",") else emptyList()

                val key = "$word|$lectId"
                if (seenKeys.add(key)) {
                    results.add(
                        GlossEntry(
                            id = id,
                            word = word,
                            lectId = lectId,
                            lectName = lectName,
                            reading = reading,
                            gloss = gloss,
                            synonyms = syns,
                            source = source,
                            dialectTags = tags,
                            isSynonymHit = false,
                        )
                    )
                }
            }
        }

        // 2. 同義詞精確反查 (Synonym keyword exact match)
        val synSql = """
            SELECT e.id, e.word, e.lect_id, e.lect_name, e.reading, e.gloss, e.synonyms, e.source, e.dialect_tags, s.keyword
            FROM synonyms s
            JOIN entries e ON s.word = e.word AND s.lect_id = e.lect_id
            WHERE s.keyword = ?
            ORDER BY $lectOrderClause
        """.trimIndent()

        db.rawQuery(synSql, arrayOf(trimmed)).use { cursor ->
            while (cursor.moveToNext()) {
                val id = cursor.getLong(0)
                val word = cursor.getString(1)
                val lectId = cursor.getString(2)
                val lectName = cursor.getString(3)
                val reading = cursor.getString(4)
                val gloss = cursor.getString(5)
                val synsStr = cursor.getString(6)
                val source = cursor.getString(7)
                val tags = cursor.getString(8)
                val keyword = cursor.getString(9)
                val syns = if (synsStr.isNotEmpty()) synsStr.split(",") else emptyList()

                val key = "$word|$lectId"
                if (seenKeys.add(key)) {
                    results.add(
                        GlossEntry(
                            id = id,
                            word = word,
                            lectId = lectId,
                            lectName = lectName,
                            reading = reading,
                            gloss = gloss,
                            synonyms = syns,
                            source = source,
                            dialectTags = tags,
                            isSynonymHit = true,
                            matchedKeyword = keyword,
                        )
                    )
                }
            }
        }

        // 3. 若結果較少，擴展同形前綴查詢 (Prefix match fallback)
        if (results.size < 5 && trimmed.length >= 2) {
            val prefixSql = """
                SELECT e.id, e.word, e.lect_id, e.lect_name, e.reading, e.gloss, e.synonyms, e.source, e.dialect_tags
                FROM entries e
                WHERE e.word LIKE ? AND e.word != ?
                ORDER BY $lectOrderClause
                LIMIT 20
            """.trimIndent()

            db.rawQuery(prefixSql, arrayOf("$trimmed%", trimmed)).use { cursor ->
                while (cursor.moveToNext()) {
                    val id = cursor.getLong(0)
                    val word = cursor.getString(1)
                    val lectId = cursor.getString(2)
                    val lectName = cursor.getString(3)
                    val reading = cursor.getString(4)
                    val gloss = cursor.getString(5)
                    val synsStr = cursor.getString(6)
                    val source = cursor.getString(7)
                    val tags = cursor.getString(8)
                    val syns = if (synsStr.isNotEmpty()) synsStr.split(",") else emptyList()

                    val key = "$word|$lectId"
                    if (seenKeys.add(key)) {
                        results.add(
                            GlossEntry(
                                id = id,
                                word = word,
                                lectId = lectId,
                                lectName = lectName,
                                reading = reading,
                                gloss = gloss,
                                synonyms = syns,
                                source = source,
                                dialectTags = tags,
                                isSynonymHit = false,
                            )
                        )
                    }
                }
            }
        }

        return results
    }
}
