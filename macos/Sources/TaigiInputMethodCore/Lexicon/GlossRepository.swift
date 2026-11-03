// Hakfa fork: SQLite repository for querying gloss.db across six Hakka lects.

import Foundation
import SQLite3

public struct GlossEntry: Equatable, Sendable {
    public let id: Int64
    public let word: String
    public let lectId: String
    public let lectName: String
    public let reading: String
    public let gloss: String
    public let synonyms: [String]
    public let source: String
    public let dialectTags: String
    public let isSynonymHit: Bool
    public let matchedKeyword: String?

    public init(
        id: Int64,
        word: String,
        lectId: String,
        lectName: String,
        reading: String,
        gloss: String,
        synonyms: [String],
        source: String,
        dialectTags: String,
        isSynonymHit: Bool = false,
        matchedKeyword: String? = nil
    ) {
        self.id = id
        self.word = word
        self.lectId = lectId
        self.lectName = lectName
        self.reading = reading
        self.gloss = gloss
        self.synonyms = synonyms
        self.source = source
        self.dialectTags = dialectTags
        self.isSynonymHit = isSynonymHit
        self.matchedKeyword = matchedKeyword
    }
}

public final class GlossRepository: @unchecked Sendable {
    public static let shared = GlossRepository()

    private var db: OpaquePointer?

    public init(databasePath: String? = nil) {
        let path = databasePath ?? resolveDatabasePath()
        if let path, FileManager.default.fileExists(atPath: path) {
            var dbPointer: OpaquePointer?
            if sqlite3_open_v2(path, &dbPointer, SQLITE_OPEN_READONLY, nil) == SQLITE_OK {
                self.db = dbPointer
            }
        }
    }

    deinit {
        if let db {
            sqlite3_close(db)
        }
    }

    private func resolveDatabasePath() -> String? {
        // 1. Bundle Resources
        if let bundlePath = Bundle.main.path(forResource: "gloss", ofType: "db") {
            return bundlePath
        }
        // 2. SPM testing / Dev environment fallback
        let candidatePaths = [
            "assets/dictionaries/gloss.db",
            "../assets/dictionaries/gloss.db",
            "../../assets/dictionaries/gloss.db",
            "../../../assets/dictionaries/gloss.db"
        ]
        for relPath in candidatePaths {
            let fullPath = (FileManager.default.currentDirectoryPath as NSString).appendingPathComponent(relPath)
            if FileManager.default.fileExists(atPath: fullPath) {
                return fullPath
            }
        }
        return nil
    }

    /// Query headwords and synonym keywords.
    /// Orders by "四南海大平安" (xiien, namxiian, hoiliug, taibu, ngiaupin, zhioon).
    public func search(query: String, lectFilter: String? = nil) -> [GlossEntry] {
        let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, let db else { return [] }

        var results: [GlossEntry] = []
        var seenKeys = Set<String>()

        let lectOrderClause: String
        if let filter = lectFilter {
            lectOrderClause = """
            CASE e.lect_id
                WHEN '\(filter)' THEN 0
                WHEN 'xiien' THEN 1
                WHEN 'namxiian' THEN 2
                WHEN 'hoiliug' THEN 3
                WHEN 'taibu' THEN 4
                WHEN 'ngiaupin' THEN 5
                WHEN 'zhioon' THEN 6
                ELSE 7
            END
            """
        } else {
            lectOrderClause = """
            CASE e.lect_id
                WHEN 'xiien' THEN 1
                WHEN 'namxiian' THEN 2
                WHEN 'hoiliug' THEN 3
                WHEN 'taibu' THEN 4
                WHEN 'ngiaupin' THEN 5
                WHEN 'zhioon' THEN 6
                ELSE 7
            END
            """
        }

        // 1. 同形詞精確比對 (Headword exact match)
        let wordSql = """
        SELECT e.id, e.word, e.lect_id, e.lect_name, e.reading, e.gloss, e.synonyms, e.source, e.dialect_tags
        FROM entries e
        WHERE e.word = ?
        ORDER BY \(lectOrderClause)
        """
        queryRows(sql: wordSql, param: trimmed, isSynonym: false, matchedKeyword: nil) { entry in
            let key = "\(entry.word)|\(entry.lectId)"
            if seenKeys.insert(key).inserted {
                results.append(entry)
            }
        }

        // 2. 同義詞精確反查 (Synonym keyword exact match)
        let synSql = """
        SELECT e.id, e.word, e.lect_id, e.lect_name, e.reading, e.gloss, e.synonyms, e.source, e.dialect_tags, s.keyword
        FROM synonyms s
        JOIN entries e ON s.word = e.word AND s.lect_id = e.lect_id
        WHERE s.keyword = ?
        ORDER BY \(lectOrderClause)
        """
        querySynonymRows(sql: synSql, param: trimmed) { entry in
            let key = "\(entry.word)|\(entry.lectId)"
            if seenKeys.insert(key).inserted {
                results.append(entry)
            }
        }

        // 3. 前綴查詢備案 (Prefix match fallback)
        if results.count < 5 && trimmed.count >= 2 {
            let prefixSql = """
            SELECT e.id, e.word, e.lect_id, e.lect_name, e.reading, e.gloss, e.synonyms, e.source, e.dialect_tags
            FROM entries e
            WHERE e.word LIKE ? AND e.word != ?
            ORDER BY \(lectOrderClause)
            LIMIT 20
            """
            queryPrefixRows(sql: prefixSql, prefix: "\(trimmed)%", exclude: trimmed) { entry in
                let key = "\(entry.word)|\(entry.lectId)"
                if seenKeys.insert(key).inserted {
                    results.append(entry)
                }
            }
        }

        return results
    }

    private func queryRows(
        sql: String,
        param: String,
        isSynonym: Bool,
        matchedKeyword: String?,
        emit: (GlossEntry) -> Void
    ) {
        guard let db else { return }
        var statement: OpaquePointer?
        guard sqlite3_prepare_v2(db, sql, -1, &statement, nil) == SQLITE_OK else { return }
        defer { sqlite3_finalize(statement) }

        sqlite3_bind_text(statement, 1, (param as NSString).utf8String, -1, nil)

        while sqlite3_step(statement) == SQLITE_ROW {
            let id = sqlite3_column_int64(statement, 0)
            let word = String(cString: sqlite3_column_text(statement, 1))
            let lectId = String(cString: sqlite3_column_text(statement, 2))
            let lectName = String(cString: sqlite3_column_text(statement, 3))
            let reading = String(cString: sqlite3_column_text(statement, 4))
            let gloss = String(cString: sqlite3_column_text(statement, 5))
            let synsStr = sqlite3_column_text(statement, 6).map { String(cString: $0) } ?? ""
            let source = String(cString: sqlite3_column_text(statement, 7))
            let dialectTags = sqlite3_column_text(statement, 8).map { String(cString: $0) } ?? ""
            let syns = synsStr.isEmpty ? [] : synsStr.components(separatedBy: ",")

            let entry = GlossEntry(
                id: id,
                word: word,
                lectId: lectId,
                lectName: lectName,
                reading: reading,
                gloss: gloss,
                synonyms: syns,
                source: source,
                dialectTags: dialectTags,
                isSynonymHit: isSynonym,
                matchedKeyword: matchedKeyword
            )
            emit(entry)
        }
    }

    private func querySynonymRows(
        sql: String,
        param: String,
        emit: (GlossEntry) -> Void
    ) {
        guard let db else { return }
        var statement: OpaquePointer?
        guard sqlite3_prepare_v2(db, sql, -1, &statement, nil) == SQLITE_OK else { return }
        defer { sqlite3_finalize(statement) }

        sqlite3_bind_text(statement, 1, (param as NSString).utf8String, -1, nil)

        while sqlite3_step(statement) == SQLITE_ROW {
            let id = sqlite3_column_int64(statement, 0)
            let word = String(cString: sqlite3_column_text(statement, 1))
            let lectId = String(cString: sqlite3_column_text(statement, 2))
            let lectName = String(cString: sqlite3_column_text(statement, 3))
            let reading = String(cString: sqlite3_column_text(statement, 4))
            let gloss = String(cString: sqlite3_column_text(statement, 5))
            let synsStr = sqlite3_column_text(statement, 6).map { String(cString: $0) } ?? ""
            let source = String(cString: sqlite3_column_text(statement, 7))
            let dialectTags = sqlite3_column_text(statement, 8).map { String(cString: $0) } ?? ""
            let keyword = String(cString: sqlite3_column_text(statement, 9))
            let syns = synsStr.isEmpty ? [] : synsStr.components(separatedBy: ",")

            let entry = GlossEntry(
                id: id,
                word: word,
                lectId: lectId,
                lectName: lectName,
                reading: reading,
                gloss: gloss,
                synonyms: syns,
                source: source,
                dialectTags: dialectTags,
                isSynonymHit: true,
                matchedKeyword: keyword
            )
            emit(entry)
        }
    }

    private func queryPrefixRows(
        sql: String,
        prefix: String,
        exclude: String,
        emit: (GlossEntry) -> Void
    ) {
        guard let db else { return }
        var statement: OpaquePointer?
        guard sqlite3_prepare_v2(db, sql, -1, &statement, nil) == SQLITE_OK else { return }
        defer { sqlite3_finalize(statement) }

        sqlite3_bind_text(statement, 1, (prefix as NSString).utf8String, -1, nil)
        sqlite3_bind_text(statement, 2, (exclude as NSString).utf8String, -1, nil)

        while sqlite3_step(statement) == SQLITE_ROW {
            let id = sqlite3_column_int64(statement, 0)
            let word = String(cString: sqlite3_column_text(statement, 1))
            let lectId = String(cString: sqlite3_column_text(statement, 2))
            let lectName = String(cString: sqlite3_column_text(statement, 3))
            let reading = String(cString: sqlite3_column_text(statement, 4))
            let gloss = String(cString: sqlite3_column_text(statement, 5))
            let synsStr = sqlite3_column_text(statement, 6).map { String(cString: $0) } ?? ""
            let source = String(cString: sqlite3_column_text(statement, 7))
            let dialectTags = sqlite3_column_text(statement, 8).map { String(cString: $0) } ?? ""
            let syns = synsStr.isEmpty ? [] : synsStr.components(separatedBy: ",")

            let entry = GlossEntry(
                id: id,
                word: word,
                lectId: lectId,
                lectName: lectName,
                reading: reading,
                gloss: gloss,
                synonyms: syns,
                source: source,
                dialectTags: dialectTags,
                isSynonymHit: false,
                matchedKeyword: nil
            )
            emit(entry)
        }
    }
}
