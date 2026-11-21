import AppKit
import CoreText
import Foundation

// Usage: swift render_mac.swift <repo_root> <trace_json> <frames_out_dir> [lect_label] [note_text]
let args = CommandLine.arguments
guard args.count >= 4 else {
    fputs("Usage: render_mac.swift <repo_root> <trace_json> <frames_out_dir> [lect_label] [note_text]\n", stderr)
    exit(1)
}

let repoRoot = URL(fileURLWithPath: args[1])
let traceURL = URL(fileURLWithPath: args[2])
let outDir = URL(fileURLWithPath: args[3])
let lectLabel = args.count >= 5 ? args[4] : "四縣腔"
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
    let cands: [CandItem]
    let committed: String
    let duration: Double
    let badge: String
}

let traceData = try Data(contentsOf: traceURL)
let traceItems = try JSONDecoder().decode([TraceItem].self, from: traceData)
let keyCount = traceItems.count

var steps: [Step] = [
    Step(preedit: "", cands: [], committed: "", duration: 0.55, badge: "Mac 版無調連打實測・\(lectLabel)")
]

for (i, item) in traceItems.enumerated() {
    let isLast = (i == traceItems.count - 1)
    let dur = isLast ? 1.55 : 0.11
    let badge = isLast ? "\(keyCount) 鍵無調連打・整句直接命中" : "Mac 版無調連打實測・\(lectLabel)"
    steps.append(Step(preedit: item.preedit, cands: item.cands, committed: "", duration: dur, badge: badge))
}

steps.append(Step(
    preedit: "",
    cands: [],
    committed: traceItems.last?.cands.first?.han ?? "",
    duration: 2.5,
    badge: "Android ＆ Mac 封測中｜hakspring.pages.dev/kipan/"
))

let _ = try? FileManager.default.removeItem(at: outDir)
try FileManager.default.createDirectory(at: outDir, withIntermediateDirectories: true)

let W: CGFloat = 1080
let H: CGFloat = 600

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

    let grad = NSGradient(colors: [color(0xF3F7EF), color(0xEAF2E3), color(0xF6EFE9)])!
    grad.draw(in: NSRect(x: 0, y: 0, width: W, height: H), angle: 35)

    let iconRect = NSRect(x: 44, y: 32, width: 46, height: 46)
    let iconShadow = NSShadow()
    iconShadow.shadowColor = color(0x4B3863, alpha: 0.18)
    iconShadow.shadowOffset = NSSize(width: 0, height: -4)
    iconShadow.shadowBlurRadius = 8
    NSGraphicsContext.saveGraphicsState()
    iconShadow.set()
    color(0xEAE3F1).setFill()
    NSBezierPath(roundedRect: iconRect, xRadius: 12, yRadius: 12).fill()
    NSGraphicsContext.restoreGraphicsState()
    color(0xD2C4E2).setStroke()
    let iconBorder = NSBezierPath(roundedRect: iconRect, xRadius: 12, yRadius: 12)
    iconBorder.lineWidth = 1.5
    iconBorder.stroke()

    let iconStr = NSAttributedString(string: "客", attributes: [
        .font: marukoFont(size: 22),
        .foregroundColor: color(0x4B3863)
    ])
    let iconSz = iconStr.size()
    iconStr.draw(at: NSPoint(x: iconRect.midX - iconSz.width / 2, y: iconRect.midY - iconSz.height / 2))

    NSAttributedString(string: "客源翠企盤 HakSpring Kipan", attributes: [
        .font: kipanFont(size: 23, bold: true),
        .foregroundColor: color(0x243326)
    ]).draw(at: NSPoint(x: 104, y: 30))

    NSAttributedString(string: "六腔客話輸入法（四南海大平安）", attributes: [
        .font: kipanFont(size: 14.5),
        .foregroundColor: color(0x5B6D5E)
    ]).draw(at: NSPoint(x: 104, y: 59))

    let badgeStr = NSAttributedString(string: step.badge, attributes: [
        .font: kipanFont(size: 15, bold: true),
        .foregroundColor: color(0x2F5233)
    ])
    let badgeSz = badgeStr.size()
    let badgeW = badgeSz.width + 36
    let badgeRect = NSRect(x: W - 44 - badgeW, y: 36, width: badgeW, height: 36)
    color(0xFFFFFF, alpha: 0.88).setFill()
    NSBezierPath(roundedRect: badgeRect, xRadius: 18, yRadius: 18).fill()
    color(0xD7E2D3).setStroke()
    NSBezierPath(roundedRect: badgeRect, xRadius: 18, yRadius: 18).stroke()
    badgeStr.draw(at: NSPoint(x: badgeRect.minX + 18, y: badgeRect.midY - badgeSz.height / 2))

    let cardRect = NSRect(x: 60, y: 122, width: 960, height: 380)
    NSGraphicsContext.saveGraphicsState()
    let cardShadow = NSShadow()
    cardShadow.shadowColor = color(0x1E3223, alpha: 0.12)
    cardShadow.shadowOffset = NSSize(width: 0, height: -12)
    cardShadow.shadowBlurRadius = 28
    cardShadow.set()
    color(0xFFFFFF, alpha: 0.97).setFill()
    NSBezierPath(roundedRect: cardRect, xRadius: 22, yRadius: 22).fill()
    NSGraphicsContext.restoreGraphicsState()
    color(0xDDE6DA).setStroke()
    NSBezierPath(roundedRect: cardRect, xRadius: 22, yRadius: 22).stroke()

    let dotColors: [UInt32] = [0xFF5F57, 0xFEBC2E, 0x28C840]
    for (i, hex) in dotColors.enumerated() {
        color(hex).setFill()
        NSBezierPath(ovalIn: NSRect(x: 94 + CGFloat(i) * 20, y: 150, width: 12, height: 12)).fill()
    }

    let textX: CGFloat = 94
    let textY: CGFloat = 200
    var cursorX = textX

    if !step.committed.isEmpty {
        let commStr = NSAttributedString(string: step.committed, attributes: [
            .font: kipanFont(size: 35),
            .foregroundColor: color(0x1C2E21)
        ])
        commStr.draw(at: NSPoint(x: textX, y: textY))
        cursorX = textX + commStr.size().width + 3
    } else if !step.preedit.isEmpty {
        let preStr = NSAttributedString(string: step.preedit, attributes: [
            .font: kipanFont(size: 30),
            .foregroundColor: color(0x1C5A3C)
        ])
        preStr.draw(at: NSPoint(x: textX, y: textY + 2))
        let pw = preStr.size().width
        color(0x1C5A3C).setFill()
        NSRect(x: textX, y: textY + 38, width: pw, height: 2.5).fill()
        cursorX = textX + pw + 2
    } else {
        let phStr = NSAttributedString(string: "準備無調連打…", attributes: [
            .font: kipanFont(size: 25),
            .foregroundColor: color(0x9AA89E)
        ])
        phStr.draw(at: NSPoint(x: textX + 10, y: textY + 4))
    }

    color(0x1C5A3C).setFill()
    NSRect(x: cursorX, y: textY + 2, width: 2.5, height: 36).fill()

    if !step.cands.isEmpty {
        struct PreparedCand {
            let numStr: NSAttributedString
            let hanStr: NSAttributedString
            let romStr: NSAttributedString
            let width: CGFloat
            let contentW: CGFloat
            let isSelected: Bool
        }

        let boxX: CGFloat = 94
        let boxY: CGFloat = 268
        let maxBarWidth: CGFloat = 880
        let arrowSpace: CGFloat = 46
        let pillH: CGFloat = 68
        let boxH: CGFloat = pillH + 12

        var prepared: [PreparedCand] = []
        var usedWidth: CGFloat = 6

        for (ci, cand) in step.cands.enumerated() {
            let isSelected = (ci == 0)
            let numColor = isSelected ? color(0xFFFFFF, alpha: 0.94) : color(0x78847B)
            let hanColor = isSelected ? color(0xFFFFFF) : color(0x202E24)
            let romColor = isSelected ? color(0xFFFFFF, alpha: 0.96) : color(0x637167)

            let numStr = NSAttributedString(string: "\(ci + 1)", attributes: [
                .font: kipanFont(size: 16, bold: true),
                .foregroundColor: numColor
            ])
            let hanStr = NSAttributedString(string: cand.han, attributes: [
                .font: kipanFont(size: 25),
                .foregroundColor: hanColor
            ])
            let romStr = NSAttributedString(string: cand.rom, attributes: [
                .font: kipanFont(size: 16),
                .foregroundColor: romColor
            ])

            let contentW = max(hanStr.size().width, romStr.size().width)
            let itemW = 14 + numStr.size().width + 12 + contentW + 18

            if ci > 0 && (usedWidth + itemW + arrowSpace > maxBarWidth) {
                break
            }
            prepared.append(PreparedCand(
                numStr: numStr,
                hanStr: hanStr,
                romStr: romStr,
                width: itemW,
                contentW: contentW,
                isSelected: isSelected
            ))
            usedWidth += itemW + 4
        }

        let boxW = usedWidth + arrowSpace
        let boxRect = NSRect(x: boxX, y: boxY, width: boxW, height: boxH)
        NSGraphicsContext.saveGraphicsState()
        let candShadow = NSShadow()
        candShadow.shadowColor = color(0x000000, alpha: 0.16)
        candShadow.shadowOffset = NSSize(width: 0, height: -10)
        candShadow.shadowBlurRadius = 24
        candShadow.set()
        color(0xFFFFFF, alpha: 0.96).setFill()
        NSBezierPath(roundedRect: boxRect, xRadius: 20, yRadius: 20).fill()
        NSGraphicsContext.restoreGraphicsState()
        color(0xD5DCD6).setStroke()
        NSBezierPath(roundedRect: boxRect, xRadius: 20, yRadius: 20).stroke()

        var curX = boxX + 6
        for (ci, p) in prepared.enumerated() {
            let itemRect = NSRect(x: curX, y: boxY + 6, width: p.width, height: pillH)
            if p.isSelected {
                let pillGrad = NSGradient(colors: [color(0xFF7E29), color(0xF46611)])!
                let pillPath = NSBezierPath(roundedRect: itemRect, xRadius: 15, yRadius: 15)
                pillGrad.draw(in: pillPath, angle: 90)
            } else if ci > 1 {
                color(0xE9EDE9).setFill()
                NSRect(x: curX - 2, y: boxY + 18, width: 1, height: boxH - 36).fill()
            }

            p.numStr.draw(at: NSPoint(x: itemRect.minX + 14, y: itemRect.midY - p.numStr.size().height / 2))
            let contentCenterX = itemRect.minX + 14 + p.numStr.size().width + 12 + p.contentW / 2
            p.hanStr.draw(at: NSPoint(x: contentCenterX - p.hanStr.size().width / 2, y: itemRect.minY + 9))
            p.romStr.draw(at: NSPoint(x: contentCenterX - p.romStr.size().width / 2, y: itemRect.minY + 40))

            curX += p.width + 4
        }

        let divX = boxX + usedWidth + 2
        color(0xE2E6E3).setFill()
        NSRect(x: divX, y: boxY + 14, width: 1, height: boxH - 28).fill()

        let arrowStr = NSAttributedString(string: "⌄", attributes: [
            .font: NSFont.systemFont(ofSize: 20, weight: .medium),
            .foregroundColor: color(0x7B847E)
        ])
        arrowStr.draw(at: NSPoint(x: divX + 14, y: boxRect.midY - arrowStr.size().height / 2 - 3))
    }

    // Optional note pill on final hit & committed frames
    if let note = noteText, idx >= steps.count - 2 {
        let noteStr = NSAttributedString(string: note, attributes: [
            .font: kipanFont(size: 15),
            .foregroundColor: color(0x556459)
        ])
        let nSz = noteStr.size()
        let pillRect = NSRect(x: 94, y: cardRect.maxY - 52, width: nSz.width + 28, height: 32)
        color(0xF2F6F0).setFill()
        NSBezierPath(roundedRect: pillRect, xRadius: 10, yRadius: 10).fill()
        color(0xDCE5D9).setStroke()
        NSBezierPath(roundedRect: pillRect, xRadius: 10, yRadius: 10).stroke()
        noteStr.draw(at: NSPoint(x: pillRect.minX + 14, y: pillRect.midY - nSz.height / 2))
    }

    let f1 = NSAttributedString(string: "客源翠企盤 ", attributes: [
        .font: kipanFont(size: 16.5),
        .foregroundColor: color(0x465A4B)
    ])
    let f2 = NSAttributedString(string: "0.0.5", attributes: [
        .font: kipanFont(size: 16.5, bold: true),
        .foregroundColor: color(0xD95A0F)
    ])
    let f3 = NSAttributedString(string: " 封測中 ｜ 免費下載試用：", attributes: [
        .font: kipanFont(size: 16.5),
        .foregroundColor: color(0x465A4B)
    ])
    let f4 = NSAttributedString(string: "hakspring.pages.dev/kipan/", attributes: [
        .font: kipanFont(size: 16.5, bold: true),
        .foregroundColor: color(0xD95A0F)
    ])
    let footer = NSMutableAttributedString()
    footer.append(f1); footer.append(f2); footer.append(f3); footer.append(f4)
    let fSz = footer.size()
    footer.draw(at: NSPoint(x: (W - fSz.width) / 2, y: H - 46))

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
print("Rendered \(steps.count) Mac frames.")
