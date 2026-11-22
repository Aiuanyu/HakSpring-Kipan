import AppKit
import CoreText
import Foundation

// Usage: swift render_android.swift <repo_root> <trace_json> <frames_out_dir> [lect_label] [note_text]
let args = CommandLine.arguments
guard args.count >= 4 else {
    fputs("Usage: render_android.swift <repo_root> <trace_json> <frames_out_dir> [lect_label] [note_text]\n", stderr)
    exit(1)
}

let repoRoot = URL(fileURLWithPath: args[1])
let traceURL = URL(fileURLWithPath: args[2])
let outDir = URL(fileURLWithPath: args[3])
let lectLabel = args.count >= 5 ? args[4] : "四縣"
let noteText = (args.count >= 6 && !args[5].isEmpty) ? args[5] : nil

let marukoURL = repoRoot.appendingPathComponent("assets/fonts/font/marukogothic_regular.ttf") as CFURL
var regError: Unmanaged<CFError>?
let _ = CTFontManagerRegisterFontsForURL(marukoURL, .process, &regError)
let marukoDesc = CTFontManagerCreateFontDescriptorsFromURL(marukoURL) as? [CTFontDescriptor]
let marukoPSName = (marukoDesc?.first.flatMap { CTFontDescriptorCopyAttribute($0, kCTFontNameAttribute) as? String }) ?? "MarukoGothicCJKtc-Regular"

func kipanFont(size: CGFloat, bold: Bool = false) -> NSFont {
    let carlitoName = bold ? "Carlito-Bold" : "Carlito"
    let baseFont = NSFont(name: carlitoName, size: size) ?? NSFont.systemFont(ofSize: size, weight: bold ? .bold : .regular)
    let fallbackDesc = CTFontDescriptorCreateWithNameAndSize(marukoPSName as CFString, size)
    let cascadeList = [fallbackDesc] as CFArray
    let attrs: [CFString: Any] = [kCTFontCascadeListAttribute: cascadeList]
    let combinedDesc = CTFontDescriptorCreateWithAttributes(attrs as CFDictionary)
    return CTFontCreateCopyWithAttributes(baseFont as CTFont, size, nil, combinedDesc) as NSFont
}

func marukoFont(size: CGFloat) -> NSFont {
    return NSFont(name: marukoPSName, size: size) ?? NSFont.systemFont(ofSize: size)
}

struct CandItem: Decodable {
    let han: String
    let rom: String
}

struct TraceItem: Decodable {
    let preedit: String
    let cands: [CandItem]
}

struct Step {
    let preedit: String
    let activeChar: String
    let cands: [CandItem]
    let committed: String
    let duration: Double
    let badge: String
}

let traceData = try Data(contentsOf: traceURL)
let traceItems = try JSONDecoder().decode([TraceItem].self, from: traceData)
let keyCount = traceItems.count

var steps: [Step] = [
    Step(preedit: "", activeChar: "", cands: [], committed: "", duration: 0.55, badge: "Android 無調連打實測・\(lectLabel)腔")
]

for (i, item) in traceItems.enumerated() {
    let isLast = (i == traceItems.count - 1)
    let ch = item.preedit.last.map { String($0) } ?? ""
    let dur = isLast ? 1.55 : (ch == "'" ? 0.50 : 0.11)
    let badge = isLast ? "\(keyCount) 鍵無調連打・整句直接命中" : "Android 無調連打實測・\(lectLabel)腔"
    steps.append(Step(preedit: item.preedit, activeChar: ch, cands: item.cands, committed: "", duration: dur, badge: badge))
}

steps.append(Step(
    preedit: "",
    activeChar: "↵",
    cands: [],
    committed: traceItems.last?.cands.first?.han ?? "",
    duration: 2.5,
    badge: "Android ＆ Mac 免費下載試用｜hakspring.pages.dev/kipan/"
))

let _ = try? FileManager.default.removeItem(at: outDir)
try FileManager.default.createDirectory(at: outDir, withIntermediateDirectories: true)

let W: CGFloat = 760
let H: CGFloat = 1040

func color(_ hex: UInt32, alpha: CGFloat = 1.0) -> NSColor {
    NSColor(
        red: CGFloat((hex >> 16) & 0xFF) / 255.0,
        green: CGFloat((hex >> 8) & 0xFF) / 255.0,
        blue: CGFloat(hex & 0xFF) / 255.0,
        alpha: alpha
    )
}

var concatText = ""

for (idx, step) in steps.enumerated() {
    let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil,
        pixelsWide: Int(W),
        pixelsHigh: Int(H),
        bitsPerSample: 8,
        samplesPerPixel: 4,
        hasAlpha: true,
        isPlanar: false,
        colorSpaceName: .deviceRGB,
        bytesPerRow: 0,
        bitsPerPixel: 0
    )!

    NSGraphicsContext.saveGraphicsState()
    let rawCtx = NSGraphicsContext(bitmapImageRep: rep)!
    let ctx = NSGraphicsContext(cgContext: rawCtx.cgContext, flipped: true)
    NSGraphicsContext.current = ctx
    let cg = ctx.cgContext
    cg.translateBy(x: 0, y: H)
    cg.scaleBy(x: 1, y: -1)

    // Soft background gradient
    let grad = NSGradient(colors: [color(0xF3F7EF), color(0xEAF2E3), color(0xF6EFE9)])!
    grad.draw(in: NSRect(x: 0, y: 0, width: W, height: H), angle: 40)

    // Header brand
    let iconRect = NSRect(x: 40, y: 24, width: 44, height: 44)
    color(0xEAE3F1).setFill()
    NSBezierPath(roundedRect: iconRect, xRadius: 11, yRadius: 11).fill()
    color(0xD2C4E2).setStroke()
    let iconBorder = NSBezierPath(roundedRect: iconRect, xRadius: 11, yRadius: 11)
    iconBorder.lineWidth = 1.5
    iconBorder.stroke()

    let iconStr = NSAttributedString(string: "客", attributes: [
        .font: marukoFont(size: 21),
        .foregroundColor: color(0x4B3863)
    ])
    let iconSz = iconStr.size()
    iconStr.draw(at: NSPoint(x: iconRect.midX - iconSz.width / 2, y: iconRect.midY - iconSz.height / 2))

    NSAttributedString(string: "客源翠企盤 HakSpring Kipan", attributes: [
        .font: kipanFont(size: 20, bold: true),
        .foregroundColor: color(0x243326)
    ]).draw(at: NSPoint(x: 96, y: 23))

    NSAttributedString(string: step.badge, attributes: [
        .font: kipanFont(size: 14, bold: true),
        .foregroundColor: color(0x2F5233)
    ]).draw(at: NSPoint(x: 96, y: 50))

    // Phone shell
    let phoneRect = NSRect(x: 56, y: 88, width: 648, height: 890)
    NSGraphicsContext.saveGraphicsState()
    let phoneShadow = NSShadow()
    phoneShadow.shadowColor = color(0x1E3223, alpha: 0.18)
    phoneShadow.shadowOffset = NSSize(width: 0, height: -14)
    phoneShadow.shadowBlurRadius = 32
    phoneShadow.set()
    color(0x232724).setFill()
    NSBezierPath(roundedRect: phoneRect, xRadius: 44, yRadius: 44).fill()
    NSGraphicsContext.restoreGraphicsState()

    // Inner screen
    let scrRect = NSRect(x: 70, y: 102, width: 620, height: 862)
    color(0xFFFFFF).setFill()
    NSBezierPath(roundedRect: scrRect, xRadius: 34, yRadius: 34).fill()

    // Status bar
    NSAttributedString(string: "14:20", attributes: [
        .font: kipanFont(size: 14, bold: true),
        .foregroundColor: color(0x333B35)
    ]).draw(at: NSPoint(x: 102, y: 118))

    color(0x1E221F).setFill()
    NSBezierPath(ovalIn: NSRect(x: W / 2 - 8, y: 116, width: 16, height: 16)).fill()

    // App top bar inside phone
    color(0xF6F8F5).setFill()
    NSRect(x: 70, y: 148, width: 620, height: 52).fill()
    color(0xE5EAE4).setFill()
    NSRect(x: 70, y: 199, width: 620, height: 1).fill()

    NSAttributedString(string: "客話記事・輸入測試", attributes: [
        .font: kipanFont(size: 18, bold: true),
        .foregroundColor: color(0x243326)
    ]).draw(at: NSPoint(x: 96, y: 162))

    // Text input area inside phone
    let textX: CGFloat = 96
    let textY: CGFloat = 232
    let maxTextW: CGFloat = 568

    if !step.committed.isEmpty {
        let commStr = NSAttributedString(string: step.committed, attributes: [
            .font: kipanFont(size: 30),
            .foregroundColor: color(0x1C2E21)
        ])
        commStr.draw(in: NSRect(x: textX, y: textY, width: maxTextW, height: 120))
        let lastX = min(textX + commStr.size().width + 3, textX + maxTextW)
        color(0x1C5A3C).setFill()
        NSRect(x: lastX, y: textY + 3, width: 2.5, height: 34).fill()
    } else if !step.preedit.isEmpty {
        let pStyle = NSMutableParagraphStyle()
        pStyle.lineBreakMode = .byCharWrapping
        pStyle.lineSpacing = 6
        let preAttr: [NSAttributedString.Key: Any] = [
            .font: kipanFont(size: 25),
            .foregroundColor: color(0x1C5A3C),
            .underlineStyle: NSUnderlineStyle.single.rawValue,
            .paragraphStyle: pStyle
        ]
        let preStr = NSAttributedString(string: step.preedit, attributes: preAttr)
        preStr.draw(in: NSRect(x: textX, y: textY, width: maxTextW, height: 110))

        let sz = preStr.size()
        if sz.width <= maxTextW {
            color(0x1C5A3C).setFill()
            NSRect(x: textX + sz.width + 2, y: textY + 3, width: 2.5, height: 30).fill()
        }
    } else {
        NSAttributedString(string: "點擊鍵盤開始無調連打…", attributes: [
            .font: kipanFont(size: 22),
            .foregroundColor: color(0x9AA89E)
        ]).draw(at: NSPoint(x: textX, y: textY + 4))
        color(0x1C5A3C).setFill()
        NSRect(x: textX, y: textY + 4, width: 2.5, height: 30).fill()
    }

    // Optional note pill above keyboard on final hit & committed frames
    if let note = noteText, idx >= steps.count - 2 {
        let noteStr = NSAttributedString(string: note, attributes: [
            .font: kipanFont(size: 14.5),
            .foregroundColor: color(0x556459)
        ])
        let nSz = noteStr.size()
        let pillRect = NSRect(x: textX, y: 452, width: nSz.width + 24, height: 32)
        color(0xF2F6F0).setFill()
        NSBezierPath(roundedRect: pillRect, xRadius: 10, yRadius: 10).fill()
        color(0xDCE5D9).setStroke()
        NSBezierPath(roundedRect: pillRect, xRadius: 10, yRadius: 10).stroke()
        noteStr.draw(at: NSPoint(x: pillRect.minX + 12, y: pillRect.midY - nSz.height / 2))
    }

    // Authentic Android default keyboard + Smartbar background (#D4D5DD from colors.xml)
    let kbTop: CGFloat = 500
    NSGraphicsContext.saveGraphicsState()
    NSBezierPath(roundedRect: scrRect, xRadius: 34, yRadius: 34).addClip()
    color(0xD4D5DD).setFill()
    NSRect(x: 70, y: kbTop, width: 620, height: 464).fill()

    // Smartbar (#D4D5DD) with left toolbar toggle button + candidates + right expand arrow
    let candBarY = kbTop
    let candBarH: CGFloat = 62

    // Left toolbar toggle icon (4-grid)
    let tbX: CGFloat = 88
    let tbY: CGFloat = candBarY + 22
    color(0x45474E).setFill()
    for r in 0..<2 {
        for c in 0..<2 {
            NSBezierPath(roundedRect: NSRect(x: tbX + CGFloat(c) * 11, y: tbY + CGFloat(r) * 11, width: 8, height: 8), xRadius: 2, yRadius: 2).fill()
        }
    }
    // Divider after toolbar toggle
    color(0xB8BAC4).setFill()
    NSRect(x: 120, y: candBarY + 16, width: 1, height: candBarH - 32).fill()

    if !step.cands.isEmpty {
        var curX: CGFloat = 130
        let maxCandX: CGFloat = 640
        for (ci, cand) in step.cands.enumerated() {
            let isFirst = (ci == 0)
            // Authentic CandidateStrip.kt: #000000 title, #6D6D73 subtitle
            let hanStr = NSAttributedString(string: cand.han, attributes: [
                .font: kipanFont(size: 21),
                .foregroundColor: color(0x000000)
            ])
            let romStr = NSAttributedString(string: cand.rom, attributes: [
                .font: kipanFont(size: 13),
                .foregroundColor: color(0x6D6D73)
            ])
            let contentW = max(hanStr.size().width, romStr.size().width)
            let itemW = max(contentW + 24, 52)

            if ci > 0 && curX + itemW > maxCandX {
                break
            }
            if isFirst {
                // Authentic first candidate fill = themeKeyBgColor (#FFFFFF)
                color(0xFFFFFF).setFill()
                NSBezierPath(roundedRect: NSRect(x: curX, y: candBarY + 6, width: itemW, height: candBarH - 12), xRadius: 8, yRadius: 8).fill()
            } else {
                color(0xB8BAC4).setFill()
                NSRect(x: curX - 3, y: candBarY + 16, width: 1, height: candBarH - 32).fill()
            }

            let centerX = curX + itemW / 2
            hanStr.draw(at: NSPoint(x: centerX - hanStr.size().width / 2, y: candBarY + 10))
            romStr.draw(at: NSPoint(x: centerX - romStr.size().width / 2, y: candBarY + 38))
            curX += itemW + 6
        }
        NSAttributedString(string: "⌄", attributes: [
            .font: kipanFont(size: 20),
            .foregroundColor: color(0x45474E)
        ]).draw(at: NSPoint(x: 656, y: candBarY + 18))
    } else {
        NSAttributedString(string: "點擊按鍵輸入客話（\(lectLabel)）", attributes: [
            .font: kipanFont(size: 15),
            .foregroundColor: color(0x555760)
        ]).draw(at: NSPoint(x: 134, y: candBarY + 24))
    }

    // 5-row Android keyboard: number row (with Hakfa tone hints) + 3 letter rows + bottom modifier row
    let digitRow = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"]
    let toneHints: [String: String] = ["2": "´", "3": "`", "5": "^", "6": "ˇ", "7": "¯"]

    let rows: [[String]] = [
        ["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
        ["a", "s", "d", "f", "g", "h", "j", "k", "l"],
        ["⇧", "z", "x", "c", "v", "b", "n", "m", "⌫"]
    ]
    let activeLower = step.activeChar.lowercased()
    let isUpper = (!step.activeChar.isEmpty && step.activeChar != activeLower)

    let kbPadX: CGFloat = 78
    let kbW: CGFloat = 604
    let keyGap: CGFloat = 6
    let digitRowH: CGFloat = 46
    let rowH: CGFloat = 56
    let rowGap: CGFloat = 8
    let digitStartY: CGFloat = candBarY + candBarH + 8
    let startY: CGFloat = digitStartY + digitRowH + rowGap

    var popupToDraw: (rect: NSRect, label: String)? = nil

    func drawKey(rect: NSRect, label: String, toneHint: String? = nil, subLabel: String? = nil, isActive: Bool = false, showPopup: Bool = false, fontSize: CGFloat = 21, muted: Bool = false) {
        let shadow = NSShadow()
        shadow.shadowColor = color(0x000000, alpha: 0.18)
        shadow.shadowOffset = NSSize(width: 0, height: -1.5)
        shadow.shadowBlurRadius = 1.0
        NSGraphicsContext.saveGraphicsState()
        shadow.set()
        // Authentic pressed color = #E0E0E0, normal & function & enter = #FFFFFF
        color(isActive ? 0xE0E0E0 : 0xFFFFFF).setFill()
        NSBezierPath(roundedRect: rect, xRadius: 7, yRadius: 7).fill()
        NSGraphicsContext.restoreGraphicsState()

        // Authentic tone diacritic overlay on digit keys (KeyContent.kt: alpha 100/255)
        if let th = toneHint {
            let thStr = NSAttributedString(string: th, attributes: [
                .font: NSFont.systemFont(ofSize: fontSize * 1.25, weight: .regular),
                .foregroundColor: color(0x000000, alpha: 0.39)
            ])
            let thSz = thStr.size()
            thStr.draw(at: NSPoint(x: rect.midX - thSz.width / 2, y: rect.minY + 3))
        }

        let txtColor = muted ? color(0x000000, alpha: 0.47) : color(0x000000)
        let str = NSAttributedString(string: label, attributes: [
            .font: kipanFont(size: fontSize),
            .foregroundColor: txtColor
        ])
        let sz = str.size()
        str.draw(at: NSPoint(x: rect.midX - sz.width / 2, y: rect.midY - sz.height / 2))

        if let sub = subLabel {
            let subStr = NSAttributedString(string: sub, attributes: [
                .font: kipanFont(size: 10),
                .foregroundColor: color(0x6D6D73)
            ])
            subStr.draw(at: NSPoint(x: rect.maxX - subStr.size().width - 5, y: rect.minY + 3))
        }

        if showPopup {
            popupToDraw = (rect, label)
        }
    }

    // Number row (1..0 with Hakfa tone marks on 2, 3, 5, 6, 7)
    let keyW10 = (kbW - keyGap * 9) / 10
    for (ci, d) in digitRow.enumerated() {
        let kx = kbPadX + CGFloat(ci) * (keyW10 + keyGap)
        let rect = NSRect(x: kx, y: digitStartY, width: keyW10, height: digitRowH)
        drawKey(rect: rect, label: d, toneHint: toneHints[d], fontSize: 18)
    }
    for (ci, k) in rows[0].enumerated() {
        let kx = kbPadX + CGFloat(ci) * (keyW10 + keyGap)
        let rect = NSRect(x: kx, y: startY, width: keyW10, height: rowH)
        let disp = isUpper ? k.uppercased() : k
        let hit = (activeLower == k)
        drawKey(rect: rect, label: disp, isActive: hit, showPopup: hit)
    }

    // Row 1
    let row1W = keyW10 * 9 + keyGap * 8
    let row1StartX = kbPadX + (kbW - row1W) / 2
    for (ci, k) in rows[1].enumerated() {
        let kx = row1StartX + CGFloat(ci) * (keyW10 + keyGap)
        let rect = NSRect(x: kx, y: startY + rowH + rowGap, width: keyW10, height: rowH)
        let disp = isUpper ? k.uppercased() : k
        let hit = (activeLower == k)
        drawKey(rect: rect, label: disp, isActive: hit, showPopup: hit)
    }

    // Row 2
    let row2Y = startY + (rowH + rowGap) * 2
    let funcW: CGFloat = (kbW - keyW10 * 7 - keyGap * 8) / 2
    drawKey(rect: NSRect(x: kbPadX, y: row2Y, width: funcW, height: rowH), label: "⇧", isActive: isUpper, fontSize: 19)
    for ci in 1...7 {
        let k = rows[2][ci]
        let kx = kbPadX + funcW + keyGap + CGFloat(ci - 1) * (keyW10 + keyGap)
        let disp = isUpper ? k.uppercased() : k
        let hit = (activeLower == k)
        drawKey(rect: NSRect(x: kx, y: row2Y, width: keyW10, height: rowH), label: disp, isActive: hit, showPopup: hit)
    }
    let bsX = kbPadX + funcW + keyGap + 7 * (keyW10 + keyGap)
    drawKey(rect: NSRect(x: bsX, y: row2Y, width: funcW, height: rowH), label: "⌫", fontSize: 18)

    // Bottom row (authentic default_halfwidth.json: 123, 🌐, ，, ☺, space, -, 漢/羅, ↵)
    let row3Y = startY + (rowH + rowGap) * 3
    let w123: CGFloat = 68
    let wGlobe: CGFloat = 46
    let wComma: CGFloat = 44
    let wEmoji: CGFloat = 46
    let wDash: CGFloat = 46
    let wTrans: CGFloat = 46
    let wEnter: CGFloat = 76
    let wSpace = kbW - w123 - wGlobe - wComma - wEmoji - wDash - wTrans - wEnter - keyGap * 7

    var rx = kbPadX
    drawKey(rect: NSRect(x: rx, y: row3Y, width: w123, height: rowH), label: "?123", fontSize: 15)
    rx += w123 + keyGap
    drawKey(rect: NSRect(x: rx, y: row3Y, width: wGlobe, height: rowH), label: "🌐", fontSize: 17)
    rx += wGlobe + keyGap
    drawKey(rect: NSRect(x: rx, y: row3Y, width: wComma, height: rowH), label: "，", fontSize: 19)
    rx += wComma + keyGap
    drawKey(rect: NSRect(x: rx, y: row3Y, width: wEmoji, height: rowH), label: "☺", fontSize: 19)
    rx += wEmoji + keyGap
    drawKey(rect: NSRect(x: rx, y: row3Y, width: wSpace, height: rowH), label: "客話（\(lectLabel)）", fontSize: 15, muted: true)
    rx += wSpace + keyGap
    let apoHit = (step.activeChar == "'")
    drawKey(rect: NSRect(x: rx, y: row3Y, width: wDash, height: rowH), label: "-", subLabel: "'", isActive: apoHit, showPopup: apoHit, fontSize: 20)
    if apoHit {
        popupToDraw = (NSRect(x: rx, y: row3Y, width: wDash, height: rowH), "'")
    }
    rx += wDash + keyGap
    drawKey(rect: NSRect(x: rx, y: row3Y, width: wTrans, height: rowH), label: "漢/羅", fontSize: 12)
    rx += wTrans + keyGap
    drawKey(rect: NSRect(x: rx, y: row3Y, width: wEnter, height: rowH), label: "↵", isActive: step.activeChar == "↵", fontSize: 20)

    // Draw authentic Android key popup bubble (#EEEEEE) or extended long-press menu above pressed key
    if let pop = popupToDraw {
        if pop.label == "'" {
            // Extended popup for '-' key from default_halfwidth.json: ' = + · – — ﹏
            let extItems = ["﹏", "—", "–", "·", "+", "=", "'"]
            let cellW: CGFloat = 46
            let cellH: CGFloat = 56
            let pad: CGFloat = 6
            let menuW = CGFloat(extItems.count) * cellW + pad * 2
            let menuH = cellH + pad * 2
            // Align rightmost selected cell (') directly above the '-' key
            let menuX = min(max(kbPadX, pop.rect.maxX - cellW - pad), scrRect.maxX - 10 - menuW)
            let menuY = pop.rect.minY - menuH - 8
            let menuRect = NSRect(x: menuX, y: menuY, width: menuW, height: menuH)

            NSGraphicsContext.saveGraphicsState()
            let popShadow = NSShadow()
            popShadow.shadowColor = color(0x000000, alpha: 0.28)
            popShadow.shadowOffset = NSSize(width: 0, height: -5)
            popShadow.shadowBlurRadius = 14
            popShadow.set()
            color(0xEEEEEE).setFill()
            NSBezierPath(roundedRect: menuRect, xRadius: 12, yRadius: 12).fill()
            NSGraphicsContext.restoreGraphicsState()

            color(0xC8C9D0).setStroke()
            NSBezierPath(roundedRect: menuRect, xRadius: 12, yRadius: 12).stroke()

            for (ei, sym) in extItems.enumerated() {
                let cellRect = NSRect(
                    x: menuX + pad + CGFloat(ei) * cellW,
                    y: menuY + pad,
                    width: cellW,
                    height: cellH
                )
                let isSelected = (sym == "'")
                if isSelected {
                    // Authentic key_popup_extended_bgColorActive (#BDBDBD)
                    color(0xBDBDBD).setFill()
                    NSBezierPath(roundedRect: cellRect, xRadius: 8, yRadius: 8).fill()
                }
                let symStr = NSAttributedString(string: sym, attributes: [
                    .font: kipanFont(size: 24, bold: isSelected),
                    .foregroundColor: color(0x000000)
                ])
                let symSz = symStr.size()
                symStr.draw(at: NSPoint(x: cellRect.midX - symSz.width / 2, y: cellRect.midY - symSz.height / 2))
            }
        } else {
            let popW = pop.rect.width + 12
            let popH: CGFloat = 68
            let popX = pop.rect.midX - popW / 2
            let popY = pop.rect.minY - popH - 6
            let popRect = NSRect(x: popX, y: popY, width: popW, height: popH)

            NSGraphicsContext.saveGraphicsState()
            let popShadow = NSShadow()
            popShadow.shadowColor = color(0x000000, alpha: 0.25)
            popShadow.shadowOffset = NSSize(width: 0, height: -4)
            popShadow.shadowBlurRadius = 10
            popShadow.set()
            color(0xEEEEEE).setFill()
            NSBezierPath(roundedRect: popRect, xRadius: 10, yRadius: 10).fill()
            NSGraphicsContext.restoreGraphicsState()

            color(0xC8C9D0).setStroke()
            NSBezierPath(roundedRect: popRect, xRadius: 10, yRadius: 10).stroke()

            let popStr = NSAttributedString(string: pop.label, attributes: [
                .font: kipanFont(size: 30, bold: true),
                .foregroundColor: color(0x000000)
            ])
            let popSz = popStr.size()
            popStr.draw(at: NSPoint(x: popRect.midX - popSz.width / 2, y: popRect.midY - popSz.height / 2))
        }
    }

    // Android gesture bar
    color(0x555E66, alpha: 0.45).setFill()
    NSBezierPath(roundedRect: NSRect(x: W / 2 - 70, y: 944, width: 140, height: 5), xRadius: 2.5, yRadius: 2.5).fill()

    NSGraphicsContext.restoreGraphicsState()

    // Bottom external CTA
    let f1 = NSAttributedString(string: "客源翠企盤 ", attributes: [
        .font: kipanFont(size: 16),
        .foregroundColor: color(0x465A4B)
    ])
    let f2 = NSAttributedString(string: "0.0.5", attributes: [
        .font: kipanFont(size: 16, bold: true),
        .foregroundColor: color(0xD95A0F)
    ])
    let f3 = NSAttributedString(string: " ｜ ", attributes: [
        .font: kipanFont(size: 16),
        .foregroundColor: color(0x465A4B)
    ])
    let f4 = NSAttributedString(string: "hakspring.pages.dev/kipan/", attributes: [
        .font: kipanFont(size: 16, bold: true),
        .foregroundColor: color(0xD95A0F)
    ])
    let footer = NSMutableAttributedString()
    footer.append(f1); footer.append(f2); footer.append(f3); footer.append(f4)
    let fSz = footer.size()
    footer.draw(at: NSPoint(x: (W - fSz.width) / 2, y: H - 42))

    NSGraphicsContext.restoreGraphicsState()

    let fileName = String(format: "frame_%02d.png", idx)
    let fileURL = outDir.appendingPathComponent(fileName)
    let pngData = rep.representation(using: .png, properties: [:])!
    try pngData.write(to: fileURL)

    concatText += "file '\(fileURL.path)'\nduration \(step.duration)\n"
}

let lastURL = outDir.appendingPathComponent(String(format: "frame_%02d.png", steps.count - 1))
concatText += "file '\(lastURL.path)'\n"
try concatText.write(to: outDir.appendingPathComponent("concat.txt"), atomically: true, encoding: .utf8)
print("Rendered \(steps.count) Android frames.")
