package com.siansiansu.taigikeyboard.ime.text.smartbar

/**
 * Hakfa fork: Converts inline dual-script Kipan annotations
 * (e.g. `字詞 (拼音)` or `拼音 (字詞)` or `<ruby>字<rt>im</rt></ruby>`)
 * into clean two-line parallel text (Hanji line + Romanization line).
 *
 * Mirrors macOS `KipanConverter.swift`.
 */
object HakfaKipanConverter {

    data class Pair(
        val han: String,
        val rom: String,
    )

    data class ParsedLine(
        val order: String,
        val hanLine: String,
        val romLine: String,
        val top: String,
        val bottom: String,
        val pairs: List<Pair>,
    )

    data class ConversionResult(
        val topText: String,
        val bottomText: String,
        val combinedText: String,
        val topLines: List<String>,
        val bottomLines: List<String>,
        val pairs: List<Pair>,
        val detectedOrder: String,
    )

    data class TargetSlice(
        val deleteBeforeLength: Int,
        val text: String,
    )

    private val HAN_REGEX = Regex("""\p{Script=Han}""")
    private val LATIN_REGEX = Regex("""[a-zA-Z\u00C0-\u024F\u1E00-\u1EFF]""")
    private val PUNCT_REGEX = Regex(
        """[，。、；：？！…‥—–―～〜·‧・／＼＋＝＜＞＆％＃＠＊＿｜｀＄＾＇＂－「」『』“”‘’《》〈〉（）〔〕【】［］｛｝〖〗〘〙〚〛〝〞﹁﹂﹃﹄﹒﹑﹐﹗﹖﹔﹕︰﹙﹚﹛﹜．｡､,\.\?!;:"'\(\)\[\]\{\}~\/\\+&%#@*_|$^]""",
    )
    private val ROM_PUNCT_SPACE_REGEX = Regex("""[,.!?;:]""")
    private val BRACKET_REGEX = Regex(
        """[\(（\[［\{｛]([^\(\)（）\[\]［］\{\}｛\}\r\n]+?)[\)）\]］\}｝]""",
    )
    private val HAN_NEUTRAL_REGEX = Regex("""^([\s\S]*?[^\p{Script=Han}\s])\s*([\p{Script=Han}]+)$""")
    private val ROM_NEUTRAL_REGEX = Regex(
        """^([0-9\.\-vV]+)\s+([a-zA-Z\u00C0-\u024F\u1E00-\u1EFF\s'’\-]+)$""",
    )
    private val NEWLINE_REGEX = Regex("""\r\n|\n|\r""")

    private val FULL_RUBY_REGEX = Regex(
        """<ruby[^>]*>(?:<rb>)?([^<]+?)(?:</rb>)?(?:\s*<rp>[^<]*</rp>\s*)?<rt[^>]*>([^<]+?)</rt>(?:\s*<rp>[^<]*</rp>\s*)?</ruby>""",
        RegexOption.IGNORE_CASE,
    )
    private val SHORT_RUBY_REGEX = Regex(
        """([^<>\s]+)<rt[^>]*>([^<]+?)</rt>""",
        RegexOption.IGNORE_CASE,
    )
    private val LEFTOVER_RUBY_REGEX = Regex(
        """</?(?:ruby|rb|rt|rp)[^>]*>""",
        RegexOption.IGNORE_CASE,
    )

    private val OPEN_PUNCT = setOf(
        '「', '『', '“', '‘', '《', '〈', '（', '〔', '【', '［', '｛',
        '〖', '〘', '〚', '〝', '﹁', '﹃', '﹙', '﹛',
        '(', '[', '{',
    )
    private val CLOSE_PUNCT = setOf(
        '」', '』', '”', '’', '》', '〉', '）', '〕', '】', '］', '｝',
        '〗', '〙', '〛', '〞', '﹂', '﹄', '﹚', '﹜',
        ')', ']', '}',
    )

    private val PUNCT_TO_ROMANCE = listOf(
        "……" to "...",
        "…" to "...",
        "‥" to "..",
        "——" to "--",
        "—" to "-",
        "–" to "-",
        "―" to "-",
        "－" to "-",
        "～" to "~",
        "〜" to "~",
        "·" to ".",
        "‧" to ".",
        "・" to ".",
        "、" to ",",
        "﹑" to ",",
        "､" to ",",
        "，" to ",",
        "﹐" to ",",
        "。" to ".",
        "．" to ".",
        "﹒" to ".",
        "｡" to ".",
        "！" to "!",
        "﹗" to "!",
        "？" to "?",
        "﹖" to "?",
        "；" to ";",
        "﹔" to ";",
        "：" to ":",
        "﹕" to ":",
        "︰" to ":",
        "（" to "(",
        "﹙" to "(",
        "）" to ")",
        "﹚" to ")",
        "【" to "[",
        "〔" to "[",
        "［" to "[",
        "〖" to "[",
        "〘" to "[",
        "〚" to "[",
        "】" to "]",
        "〕" to "]",
        "］" to "]",
        "〗" to "]",
        "〙" to "]",
        "〛" to "]",
        "｛" to "{",
        "﹛" to "{",
        "｝" to "}",
        "﹜" to "}",
        "「" to "\"",
        "“" to "\"",
        "《" to "\"",
        "〝" to "\"",
        "＂" to "\"",
        "﹁" to "\"",
        "」" to "\"",
        "”" to "\"",
        "》" to "\"",
        "〞" to "\"",
        "﹂" to "\"",
        "『" to "'",
        "‘" to "'",
        "〈" to "'",
        "＇" to "'",
        "﹃" to "'",
        "』" to "'",
        "’" to "'",
        "〉" to "'",
        "﹄" to "'",
        "／" to "/",
        "＼" to "\\",
        "＋" to "+",
        "＝" to "=",
        "＜" to "<",
        "＞" to ">",
        "＆" to "&",
        "％" to "%",
        "＃" to "#",
        "＠" to "@",
        "＊" to "*",
        "＿" to "_",
        "｜" to "|",
        "｀" to "`",
        "＄" to "$",
        "＾" to "^",
    )

    private val PUNCT_TO_HAN = listOf(
        "..." to "……",
        "--" to "——",
        "~" to "～",
        "," to "，",
        "." to "。",
        "!" to "！",
        "?" to "？",
        ";" to "；",
        ":" to "：",
        "(" to "（",
        ")" to "）",
        "[" to "［",
        "]" to "］",
        "{" to "｛",
        "}" to "｝",
        "/" to "／",
        "\\" to "＼",
        "+" to "＋",
        "=" to "＝",
        "&" to "＆",
        "%" to "％",
        "#" to "＃",
        "@" to "＠",
        "*" to "＊",
        "_" to "＿",
        "|" to "｜",
        "$" to "＄",
        "^" to "＾",
    )

    fun hasHan(str: String): Boolean = HAN_REGEX.containsMatchIn(str)

    fun hasLatin(str: String): Boolean = LATIN_REGEX.containsMatchIn(str)

    fun convertPunctToRomance(p: String): String {
        if (p.isEmpty()) return ""
        var result = p
        for ((target, rep) in PUNCT_TO_ROMANCE) {
            result = result.replace(target, rep)
        }
        return result
    }

    fun convertPunctToHan(p: String): String {
        if (p.isEmpty()) return ""
        var result = p
        for ((target, rep) in PUNCT_TO_HAN) {
            result = result.replace(target, rep)
        }
        return result
    }

    data class SplitBetweenResult(
        val prevPunct: String,
        val currOpen: String,
        val core: String,
        val currClose: String,
        val trailingSpace: String,
    )

    fun splitBetween(between: String): SplitBetweenResult {
        if (between.isEmpty()) {
            return SplitBetweenResult("", "", "", "", "")
        }

        fun isPunctOrSpace(ch: Char): Boolean {
            if (ch.isWhitespace()) return true
            return PUNCT_REGEX.matches(ch.toString())
        }

        var start = 0
        while (start < between.length && isPunctOrSpace(between[start])) {
            start++
        }

        var end = between.length
        while (end > start && isPunctOrSpace(between[end - 1])) {
            end--
        }

        val leading = between.substring(0, start)
        val core = between.substring(start, end)
        val trailing = between.substring(end)

        var openIdx = leading.length
        while (openIdx > 0 && leading[openIdx - 1] in OPEN_PUNCT) {
            openIdx--
        }
        val prevPunct = leading.substring(0, openIdx)
        val currOpen = leading.substring(openIdx)

        var closeIdx = 0
        while (closeIdx < trailing.length && trailing[closeIdx] in CLOSE_PUNCT) {
            closeIdx++
        }
        val currClose = trailing.substring(0, closeIdx)
        val trailingSpace = trailing.substring(closeIdx)

        return SplitBetweenResult(prevPunct, currOpen, core, currClose, trailingSpace)
    }

    fun normalizeRuby(text: String): String {
        if (!text.contains("<rt", ignoreCase = true) && !text.contains("<ruby", ignoreCase = true)) {
            return text
        }
        var result = FULL_RUBY_REGEX.replace(text) { m -> "${m.groupValues[1]} (${m.groupValues[2]})" }
        result = SHORT_RUBY_REGEX.replace(result) { m -> "${m.groupValues[1]} (${m.groupValues[2]})" }
        result = LEFTOVER_RUBY_REGEX.replace(result, "")
        return result
    }

    fun parseKipanLine(
        rawLine: String,
        forceOrder: String = "auto",
    ): ParsedLine {
        val line = normalizeRuby(rawLine)
        val matches = BRACKET_REGEX.findAll(line).toList()
        if (matches.isEmpty()) {
            return ParsedLine(
                order = "han-top",
                hanLine = line,
                romLine = line,
                top = line,
                bottom = line,
                pairs = emptyList(),
            )
        }

        data class TokenItem(val between: String, val inside: String)

        val tokens = ArrayList<TokenItem>(matches.size)
        var lastEnd = 0
        for (m in matches) {
            val between = line.substring(lastEnd, m.range.first)
            val inside = m.groupValues[1]
            tokens.add(TokenItem(between, inside))
            lastEnd = m.range.last + 1
        }
        val trailing = line.substring(lastEnd)

        var isHanOutside = true
        for (tok in tokens) {
            if (hasHan(tok.between)) {
                isHanOutside = true
                break
            }
            if (hasLatin(tok.between) && hasHan(tok.inside)) {
                isHanOutside = false
                break
            }
        }

        val hanLine = StringBuilder()
        val romLine = StringBuilder()
        val pairs = ArrayList<Pair>(tokens.size)

        for (i in tokens.indices) {
            val tok = tokens[i]
            val (prevPunct, currOpen, core, currClose, _) = splitBetween(tok.between)
            val inside = tok.inside

            if (prevPunct.isNotEmpty()) {
                val p = prevPunct.replace(Regex("\\s+"), "")
                if (p.isNotEmpty()) {
                    hanLine.append(convertPunctToHan(p))
                    val romP = convertPunctToRomance(p)
                    romLine.append(romP)
                    if (!romLine.endsWith(" ") && ROM_PUNCT_SPACE_REGEX.containsMatchIn(romP)) {
                        romLine.append(' ')
                    }
                } else if (i > 0 && !romLine.endsWith(" ")) {
                    romLine.append(' ')
                }
            } else if (i > 0 && !romLine.endsWith(" ")) {
                romLine.append(' ')
            }

            if (currOpen.isNotEmpty()) {
                hanLine.append(convertPunctToHan(currOpen))
                romLine.append(convertPunctToRomance(currOpen))
            }

            var neutral = ""
            var effectiveCore = core

            if (isHanOutside) {
                val match = HAN_NEUTRAL_REGEX.matchEntire(core)
                if (match != null) {
                    val neutralPart = match.groupValues[1].trim()
                    val corePart = match.groupValues[2]
                    if (neutralPart.isNotEmpty()) {
                        neutral = neutralPart
                        effectiveCore = corePart
                    }
                }
            } else {
                val match = ROM_NEUTRAL_REGEX.matchEntire(core)
                if (match != null) {
                    val neutralPart = match.groupValues[1].trim()
                    val corePart = match.groupValues[2]
                    if (neutralPart.isNotEmpty()) {
                        neutral = neutralPart
                        effectiveCore = corePart
                    }
                }
            }

            if (neutral.isNotEmpty()) {
                if (hanLine.isNotEmpty() && !hanLine.endsWith(" ")) hanLine.append(' ')
                hanLine.append(neutral).append(' ')
                if (romLine.isNotEmpty() && !romLine.endsWith(" ")) romLine.append(' ')
                romLine.append(neutral).append(' ')
            }

            val hanWord: String
            val romWord: String
            if (isHanOutside) {
                hanWord = effectiveCore
                romWord = inside
            } else {
                hanWord = inside
                romWord = effectiveCore
            }

            hanLine.append(hanWord)
            romLine.append(romWord)

            if (currClose.isNotEmpty()) {
                hanLine.append(convertPunctToHan(currClose))
                romLine.append(convertPunctToRomance(currClose))
            }

            pairs.add(Pair(hanWord, romWord))
        }

        if (trailing.isNotEmpty()) {
            val tTrim = trailing.trim()
            if (tTrim.isNotEmpty()) {
                hanLine.append(convertPunctToHan(tTrim))
                romLine.append(convertPunctToRomance(tTrim))
            }
        }

        val finalOrder = if (forceOrder == "auto") {
            if (isHanOutside) "han-top" else "rom-top"
        } else {
            forceOrder
        }

        val hanResult = hanLine.toString().trim()
        val romResult = romLine.toString().trim()
        val top = if (finalOrder == "han-top") hanResult else romResult
        val bottom = if (finalOrder == "han-top") romResult else hanResult

        return ParsedLine(
            order = finalOrder,
            hanLine = hanResult,
            romLine = romResult,
            top = top,
            bottom = bottom,
            pairs = pairs,
        )
    }

    fun convertKipanText(
        text: String,
        forceOrder: String = "auto",
    ): ConversionResult {
        if (text.isEmpty()) {
            return ConversionResult(
                topText = "",
                bottomText = "",
                combinedText = "",
                topLines = emptyList(),
                bottomLines = emptyList(),
                pairs = emptyList(),
                detectedOrder = "han-top",
            )
        }

        val lines = text.split(NEWLINE_REGEX)
        val topLines = ArrayList<String>(lines.size)
        val bottomLines = ArrayList<String>(lines.size)
        var detectedOrder = "han-top"
        val allPairs = ArrayList<Pair>()

        for ((i, line) in lines.withIndex()) {
            if (line.trim().isEmpty()) {
                topLines.add("")
                bottomLines.add("")
                continue
            }

            val res = parseKipanLine(line, forceOrder = forceOrder)
            if (i == 0) {
                detectedOrder = res.order
            }
            topLines.add(res.top)
            bottomLines.add(res.bottom)
            allPairs.addAll(res.pairs)
        }

        val topText = topLines.joinToString("\n")
        val bottomText = bottomLines.joinToString("\n")
        val combinedText = "$topText\n$bottomText"

        return ConversionResult(
            topText = topText,
            bottomText = bottomText,
            combinedText = combinedText,
            topLines = topLines,
            bottomLines = bottomLines,
            pairs = allPairs,
            detectedOrder = detectedOrder,
        )
    }

    /**
     * Picks the best target slice from text before the cursor when no text is selected:
     * prefers the current paragraph (after the last newline) if it contains inline annotations,
     * otherwise falls back to the full [textBeforeCursor] if it contains inline annotations.
     */
    fun extractConvertibleTarget(textBeforeCursor: String): TargetSlice? {
        if (textBeforeCursor.isEmpty()) return null
        val lastNewline = textBeforeCursor.lastIndexOf('\n')
        val currentParagraph = if (lastNewline >= 0) {
            textBeforeCursor.substring(lastNewline + 1)
        } else {
            textBeforeCursor
        }
        if (currentParagraph.isNotEmpty() && convertKipanText(currentParagraph).pairs.isNotEmpty()) {
            return TargetSlice(deleteBeforeLength = currentParagraph.length, text = currentParagraph)
        }
        if (lastNewline >= 0 && convertKipanText(textBeforeCursor).pairs.isNotEmpty()) {
            return TargetSlice(deleteBeforeLength = textBeforeCursor.length, text = textBeforeCursor)
        }
        return null
    }
}
