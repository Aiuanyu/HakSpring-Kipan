// KipanConverter.swift
// 客源翠企盤（HakSpring-Kipan）輸入法並列格式轉換器
//
// 將企盤輸出的「字詞 (拼音)」或「拼音 (字詞)」混排文字，
// 自動分離轉換為整齊的兩行（諸夏字行與羅馬字行）。
// 頂層行依第一個有效字詞自動判定（漢字在先則諸夏字在頂，羅馬字在先則羅馬字在頂）。

import Foundation

public enum KipanConverter {

    public struct Pair: Equatable, Sendable {
        public let han: String
        public let rom: String

        public init(han: String, rom: String) {
            self.han = han
            self.rom = rom
        }
    }

    public struct ParsedLine: Equatable, Sendable {
        public let order: String
        public let hanLine: String
        public let romLine: String
        public let top: String
        public let bottom: String
        public let pairs: [Pair]

        public init(order: String, hanLine: String, romLine: String, top: String, bottom: String, pairs: [Pair]) {
            self.order = order
            self.hanLine = hanLine
            self.romLine = romLine
            self.top = top
            self.bottom = bottom
            self.pairs = pairs
        }
    }

    public struct ConversionResult: Equatable, Sendable {
        public let topText: String
        public let bottomText: String
        public let combinedText: String
        public let topLines: [String]
        public let bottomLines: [String]
        public let pairs: [Pair]
        public let detectedOrder: String

        public init(topText: String, bottomText: String, combinedText: String, topLines: [String], bottomLines: [String], pairs: [Pair], detectedOrder: String) {
            self.topText = topText
            self.bottomText = bottomText
            self.combinedText = combinedText
            self.topLines = topLines
            self.bottomLines = bottomLines
            self.pairs = pairs
            self.detectedOrder = detectedOrder
        }
    }

    /// 判斷是否包含漢字
    public static func hasHan(_ str: String) -> Bool {
        return str.range(of: #"\p{Script=Han}"#, options: .regularExpression) != nil
    }

    /// 判斷是否包含拉丁字母/羅馬字
    public static func hasLatin(_ str: String) -> Bool {
        return str.range(of: #"[a-zA-Z\u00C0-\u024F\u1E00-\u1EFF]"#, options: .regularExpression) != nil
    }

    /// 中文全形標點轉羅馬字半形標點
    /// 特別處理：頓號「、」轉「,」，句號「。」轉「.」，各類括號「（）【】」轉「()[]」，刪節號轉「...」等
    public static func convertPunctToRomance(_ p: String) -> String {
        if p.isEmpty { return "" }
        var result = p
        let replacements: [(String, String)] = [
            ("……", "..."),
            ("…", "..."),
            ("——", "--"),
            ("、", ","),
            ("，", ","),
            ("。", "."),
            ("！", "!"),
            ("？", "?"),
            ("；", ";"),
            ("：", ":"),
            ("（", "("),
            ("）", ")"),
            ("【", "["), ("〔", "["), ("［", "["),
            ("】", "]"), ("〕", "]"), ("］", "]"),
            ("｛", "{"),
            ("｝", "}"),
            ("「", "\""), ("“", "\""), ("《", "\""),
            ("」", "\""), ("”", "\""), ("》", "\""),
            ("『", "'"), ("‘", "'"), ("〈", "'"),
            ("』", "'"), ("’", "'"), ("〉", "'")
        ]
        for (target, rep) in replacements {
            result = result.replacingOccurrences(of: target, with: rep)
        }
        return result
    }

    /// 羅馬字半形標點轉中文全形標點
    /// 包含逗號轉「，」、句號轉「。」、問驚號、各類半形括號轉全形括號等雙向轉換
    public static func convertPunctToHan(_ p: String) -> String {
        if p.isEmpty { return "" }
        var result = p
        let replacements: [(String, String)] = [
            ("...", "……"),
            ("--", "——"),
            (",", "，"),
            (".", "。"),
            ("!", "！"),
            ("?", "？"),
            (";", "；"),
            (":", "："),
            ("(", "（"),
            (")", "）"),
            ("[", "［"),
            ("]", "］"),
            ("{", "｛"),
            ("}", "｝")
        ]
        for (target, rep) in replacements {
            result = result.replacingOccurrences(of: target, with: rep)
        }
        return result
    }

    private static let openPunct: Set<Character> = ["「", "『", "“", "‘", "《", "〈", "（", "〔", "【", "［", "｛", "(", "[", "{"]
    private static let closePunct: Set<Character> = ["」", "』", "”", "’", "》", "〉", "）", "〕", "】", "］", "｝", ")", "]", "}"]
    private static let punctPattern = #"[，。、；：？！…—～「」『』“”‘’《》〈〉（）〔〕【】［］｛｝,\.\?!;:"'\(\)\[\]\{\}]"#

    /// 拆分詞與詞之間的字串（between），精細分離出：
    /// 1. 前詞結尾標點（prevPunct）
    /// 2. 當前詞開頭標點/括號（currOpen）
    /// 3. 當前詞核心字詞（core）
    /// 4. 當前詞結尾標點/括號（currClose）
    /// 5. 尾隨空白（trailingSpace）
    public static func splitBetween(_ between: String) -> (prevPunct: String, currOpen: String, core: String, currClose: String, trailingSpace: String) {
        guard !between.isEmpty else {
            return ("", "", "", "", "")
        }

        let chars = Array(between)
        let punctRegex = try? NSRegularExpression(pattern: punctPattern)

        func isPunctOrSpace(_ ch: Character) -> Bool {
            if ch.isWhitespace { return true }
            let s = String(ch)
            let range = NSRange(location: 0, length: s.utf16.count)
            return punctRegex?.firstMatch(in: s, options: [], range: range) != nil
        }

        var start = 0
        while start < chars.count && isPunctOrSpace(chars[start]) {
            start += 1
        }

        var end = chars.count
        while end > start && isPunctOrSpace(chars[end - 1]) {
            end -= 1
        }

        let leadingChars = chars[0..<start]
        let core = String(chars[start..<end])
        let trailingChars = chars[end..<chars.count]

        var openIdx = leadingChars.count
        while openIdx > 0 && openPunct.contains(leadingChars[openIdx - 1]) {
            openIdx -= 1
        }
        let prevPunct = String(leadingChars[0..<openIdx])
        let currOpen = String(leadingChars[openIdx..<leadingChars.count])

        var closeIdx = 0
        while closeIdx < trailingChars.count && closePunct.contains(trailingChars[trailingChars.startIndex + closeIdx]) {
            closeIdx += 1
        }
        let currClose = String(trailingChars[trailingChars.startIndex..<(trailingChars.startIndex + closeIdx)])
        let trailingSpace = String(trailingChars[(trailingChars.startIndex + closeIdx)..<trailingChars.endIndex])

        return (prevPunct, currOpen, core, currClose, trailingSpace)
    }

    /// 將 HTML ruby 標註（<ruby>...<rt>...</rt></ruby> 或 ...<rt>...</rt>）正規化為括號標註格式
    public static func normalizeRuby(_ text: String) -> String {
        guard text.contains("<rt") || text.contains("<ruby") else { return text }
        var result = text

        // 1. 標準 <ruby> 標籤模式：<ruby>漢<rt>han</rt></ruby> 或含 <rp>、<rb>
        let fullRubyPattern = #"<ruby[^>]*>(?:<rb>)?([^<]+?)(?:<\/rb>)?(?:\s*<rp>[^<]*<\/rp>\s*)?<rt[^>]*>([^<]+?)<\/rt>(?:\s*<rp>[^<]*<\/rp>\s*)?<\/ruby>"#
        if let regex = try? NSRegularExpression(pattern: fullRubyPattern, options: [.caseInsensitive]) {
            result = regex.stringByReplacingMatches(
                in: result,
                options: [],
                range: NSRange(location: 0, length: (result as NSString).length),
                withTemplate: "$1 ($2)"
            )
        }

        // 2. 簡寫模式：字詞<rt>拼音</rt>
        let shortRubyPattern = #"([^<>\s]+)<rt[^>]*>([^<]+?)<\/rt>"#
        if let regex = try? NSRegularExpression(pattern: shortRubyPattern, options: [.caseInsensitive]) {
            result = regex.stringByReplacingMatches(
                in: result,
                options: [],
                range: NSRange(location: 0, length: (result as NSString).length),
                withTemplate: "$1 ($2)"
            )
        }

        // 3. 清除殘留的 ruby 相關標籤
        let leftoverPattern = #"<\/?(?:ruby|rb|rt|rp)[^>]*>"#
        if let regex = try? NSRegularExpression(pattern: leftoverPattern, options: [.caseInsensitive]) {
            result = regex.stringByReplacingMatches(
                in: result,
                options: [],
                range: NSRange(location: 0, length: (result as NSString).length),
                withTemplate: ""
            )
        }

        return result
    }

    /// 解析單行企盤格式文字
    public static func parseKipanLine(_ rawLine: String, autoSpaceRomance: Bool = false, forceOrder: String = "auto") -> ParsedLine {
        // 先將 ruby/rt 標籤正規化為括號標註
        let line = normalizeRuby(rawLine)

        // 1. 找出所有標註括號區塊 (支援圓括號 ()、（）、方括號 []、［］ 與花括號 {}、｛｝)
        let bracketRegex = try! NSRegularExpression(pattern: #"[\(（\[［\{｛]([^\(\)（）\[\]［］\{\}｛\}\r\n]+?)[\)）\]］\}｝]"#)
        let nsLine = line as NSString
        let matches = bracketRegex.matches(in: line, options: [], range: NSRange(location: 0, length: nsLine.length))

        // 若無括號結構，直接原樣回傳
        if matches.isEmpty {
            return ParsedLine(
                order: "han-top",
                hanLine: line,
                romLine: line,
                top: line,
                bottom: line,
                pairs: []
            )
        }

        // 2. 切分各個 segment
        struct MatchItem {
            let start: Int
            let end: Int
            let inside: String
        }

        var matchItems: [MatchItem] = []
        for m in matches {
            let insideRange = m.range(at: 1)
            let inside = nsLine.substring(with: insideRange)
            matchItems.append(MatchItem(start: m.range.location, end: m.range.location + m.range.length, inside: inside))
        }

        struct TokenItem {
            let between: String
            let inside: String
        }

        var tokens: [TokenItem] = []
        var lastEnd = 0

        for item in matchItems {
            let betweenRange = NSRange(location: lastEnd, length: item.start - lastEnd)
            let between = nsLine.substring(with: betweenRange)
            tokens.append(TokenItem(between: between, inside: item.inside))
            lastEnd = item.end
        }

        let trailingRange = NSRange(location: lastEnd, length: nsLine.length - lastEnd)
        let trailing = nsLine.substring(with: trailingRange)

        // 3. 判斷輸入文字的本體結構：是漢字在括號外、羅馬字在括號內（漢外羅內），還是反之
        var isHanOutside = true
        for tok in tokens {
            if hasHan(tok.between) {
                isHanOutside = true
                break
            }
            if hasLatin(tok.between) && hasHan(tok.inside) {
                isHanOutside = false
                break
            }
        }

        var hanLine = ""
        var romLine = ""
        var pairs: [Pair] = []

        for i in 0..<tokens.count {
            let tok = tokens[i]
            let (prevPunct, currOpen, core, currClose, _) = splitBetween(tok.between)
            let inside = tok.inside

            // 處理前詞標點 (prevPunct) 與空格
            if !prevPunct.isEmpty {
                let p = prevPunct.trimmingCharacters(in: .whitespacesAndNewlines)
                if !p.isEmpty {
                    hanLine += convertPunctToHan(p)
                    let romP = convertPunctToRomance(p)
                    romLine += romP
                    if !romLine.hasSuffix(" ") && romP.range(of: #"[,.!?;:]"#, options: .regularExpression) != nil {
                        romLine += " "
                    }
                } else if i > 0 && !romLine.hasSuffix(" ") {
                    romLine += " "
                }
            } else if i > 0 && !romLine.hasSuffix(" ") {
                romLine += " "
            }

            // 處理本詞開頭標點/括號 (currOpen)
            if !currOpen.isEmpty {
                hanLine += convertPunctToHan(currOpen)
                romLine += convertPunctToRomance(currOpen)
            }

            var hanWord = ""
            var romWord = ""

            // 檢查 core 中是否包含未標註的中立字詞（如英數字、版本號 v0.0.5、ASCII 等）
            var neutral = ""
            var effectiveCore = core

            if isHanOutside {
                // 漢字在括號外：核心詞是漢字（[\p{Script=Han}]+），前面若夾帶英數字/符號（如 v0.0.5、100）則為 neutral
                let pattern = #"^([\s\S]*?[^\p{Script=Han}\s])\s*([\p{Script=Han}]+)$"#
                if let regex = try? NSRegularExpression(pattern: pattern),
                   let match = regex.firstMatch(in: core, options: [], range: NSRange(location: 0, length: (core as NSString).length)) {
                    let nsCore = core as NSString
                    let neutralPart = nsCore.substring(with: match.range(at: 1)).trimmingCharacters(in: .whitespacesAndNewlines)
                    let corePart = nsCore.substring(with: match.range(at: 2))
                    if !neutralPart.isEmpty {
                        neutral = neutralPart
                        effectiveCore = corePart
                    }
                }
            } else {
                // 羅馬字在括號外：只有當開頭為純數字或版本號時才分離 neutral，避免拆散有縫仔拼音（如 gì oi）
                let pattern = #"^([0-9\.\-vV]+)\s+([a-zA-Z\u00C0-\u024F\u1E00-\u1EFF\s'’\-]+)$"#
                if let regex = try? NSRegularExpression(pattern: pattern),
                   let match = regex.firstMatch(in: core, options: [], range: NSRange(location: 0, length: (core as NSString).length)) {
                    let nsCore = core as NSString
                    let neutralPart = nsCore.substring(with: match.range(at: 1)).trimmingCharacters(in: .whitespacesAndNewlines)
                    let corePart = nsCore.substring(with: match.range(at: 2))
                    if !neutralPart.isEmpty {
                        neutral = neutralPart
                        effectiveCore = corePart
                    }
                }
            }

            // 若有中立字詞（如 v0.0.5），兩行並列保留
            if !neutral.isEmpty {
                if !hanLine.isEmpty && !hanLine.hasSuffix(" ") { hanLine += " " }
                hanLine += neutral + " "
                if !romLine.isEmpty && !romLine.hasSuffix(" ") { romLine += " " }
                romLine += neutral + " "
            }

            if isHanOutside {
                hanWord = effectiveCore
                romWord = inside
            } else {
                hanWord = inside
                romWord = effectiveCore
            }

            hanLine += hanWord
            romLine += romWord

            // 處理本詞結尾標點/括號 (currClose)
            if !currClose.isEmpty {
                hanLine += convertPunctToHan(currClose)
                romLine += convertPunctToRomance(currClose)
            }

            pairs.append(Pair(han: hanWord, rom: romWord))
        }

        // 處理尾隨標點與字串
        if !trailing.isEmpty {
            let tTrim = trailing.trimmingCharacters(in: .whitespacesAndNewlines)
            if !tTrim.isEmpty {
                hanLine += convertPunctToHan(tTrim)
                romLine += convertPunctToRomance(tTrim)
            }
        }

        // 判定最終頂層與底層的排版位置
        var finalOrder = forceOrder
        if finalOrder == "auto" {
            finalOrder = isHanOutside ? "han-top" : "rom-top"
        }

        let top = (finalOrder == "han-top" ? hanLine : romLine).trimmingCharacters(in: .whitespacesAndNewlines)
        let bottom = (finalOrder == "han-top" ? romLine : hanLine).trimmingCharacters(in: .whitespacesAndNewlines)

        return ParsedLine(
            order: finalOrder,
            hanLine: hanLine.trimmingCharacters(in: .whitespacesAndNewlines),
            romLine: romLine.trimmingCharacters(in: .whitespacesAndNewlines),
            top: top,
            bottom: bottom,
            pairs: pairs
        )
    }

    /// 轉換整段企盤文字（支援多行段落）
    public static func convertKipanText(_ text: String, autoSpaceRomance: Bool = false, forceOrder: String = "auto") -> ConversionResult {
        guard !text.isEmpty else {
            return ConversionResult(
                topText: "",
                bottomText: "",
                combinedText: "",
                topLines: [],
                bottomLines: [],
                pairs: [],
                detectedOrder: "han-top"
            )
        }

        let lines = text.components(separatedBy: .newlines)
        var topLines: [String] = []
        var bottomLines: [String] = []
        var detectedOrder = "han-top"
        var allPairs: [Pair] = []

        for (i, line) in lines.enumerated() {
            if line.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                topLines.append("")
                bottomLines.append("")
                continue
            }

            let res = parseKipanLine(line, autoSpaceRomance: autoSpaceRomance, forceOrder: forceOrder)
            if i == 0 {
                detectedOrder = res.order
            }
            topLines.append(res.top)
            bottomLines.append(res.bottom)
            allPairs.append(contentsOf: res.pairs)
        }

        let topText = topLines.joined(separator: "\n")
        let bottomText = bottomLines.joined(separator: "\n")
        let combinedText = "\(topText)\n\(bottomText)"

        return ConversionResult(
            topText: topText,
            bottomText: bottomText,
            combinedText: combinedText,
            topLines: topLines,
            bottomLines: bottomLines,
            pairs: allPairs,
            detectedOrder: detectedOrder
        )
    }
}
