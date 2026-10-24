// 【備用・舊版：「客」字圖示】企盤 Android 圖示已改成構想 3-1（鍵帽 × h，方案 3A），
// 產生程式是 tools/hakfa/build-kipan-icons.py。這支和那支寫**同一批檔案**
// （mipmap-*/ic_launcher*.webp、ic_launcher-playstore.png）：執行這支會把目前的圖示蓋成「客」字版，
// 而且不產生 ic_launcher_monochrome.webp（主題圖示會壞）。要用先備份，或改輸出路徑。
// 平哥 20261024 決定留著備用。
//
// Generates the Android launcher icons from the same 客 outline as the macOS
// app icon (tools/desktop/make-app-icon.swift), so the platforms cannot drift:
//
// - the adaptive icon foreground — android/app/src/main/res/mipmap-*/
//   ic_launcher_foreground.webp, a transparent 108 dp canvas with 客 inside the
//   66 dp safe zone. The background is res/values/ic_launcher_background.xml
//   (pale purple #E6DEEC, same as the tile) and the monochrome layer reuses the
//   foreground (the system only reads its alpha), see
//   res/mipmap-anydpi-v26/ic_launcher.xml — neither is touched here;
// - the legacy icons (Android 7 and below; minSdk is 28, so they are a
//   fallback) — ic_launcher.webp (rounded square, like the macOS tile) and
//   ic_launcher_round.webp (circle);
// - the Google Play listing icon — android/app/src/main/ic_launcher-playstore.png,
//   512 × 512, full-bleed (Play applies its own mask).
//
// Run it by hand after changing anything below:
//
//     swift tools/hakfa/make-android-icon.swift
//     swift tools/hakfa/make-android-icon.swift --check   (verify, write nothing)
//
// Requires macOS (CoreText) and `cwebp` (brew install webp). Every output is a
// committed artefact: do NOT wire this into the Gradle build — generating
// during a build would let the build machine's font and rasteriser decide what
// ships. iOS keeps the "Tâi" wordmark and is not touched here.

import AppKit
import CoreText
import CryptoKit
import ImageIO

// MARK: - The design

let glyph = "客"
let fontPostScriptName = "ZenMaruGothic-Bold"
/// The same outline fingerprint make-app-icon.swift approved for 客. Regenerating
/// fails loudly if the font changed under a stable name.
let approvedFingerprint = "13f91b78561babf8051e8919b34d16038a610a0277a4cebdd61918d2c90e9669"

/// HakSpring palette: pale purple tile (a tint of #AA96B7) and deep green ink (#245D48).
let tileColor = (red: 0xE6 / 255.0, green: 0xDE / 255.0, blue: 0xEC / 255.0)
let inkColor = (red: 0x24 / 255.0, green: 0x5D / 255.0, blue: 0x48 / 255.0)

/// Adaptive icon: the canvas is 108 dp and a launcher may mask it down to a
/// circle, so the guaranteed area is the circle of diameter 66 dp in the
/// centre (radius 33 dp). 客 is wide and square-ish, so its ink reaches into
/// the corners of its own box: at 60 % of the canvas the farthest ink was
/// 36.1 dp from the centre, outside that circle. At 54 % it is 32.5 dp, inside.
/// `checkSafeZone` below measures it on every run rather than trusting this.
let adaptiveGlyphRatio: CGFloat = 0.54
let safeZoneRadiusDp: CGFloat = 33
/// The macOS tile's inset and corner radius, for the legacy rounded square.
let legacySquareGlyphRatio: CGFloat = 1 - 0.13 * 2
let legacySquareCornerRatio: CGFloat = 3.0 / 16.0
/// The legacy round icon is a plain circle of the whole 48 dp page; at 70 %
/// the farthest ink is about 0.39 of the page from the centre, inside the 0.5
/// radius with a margin.
let legacyRoundGlyphRatio: CGFloat = 0.70
/// Play lists the icon at 512 × 512 and masks it itself: full-bleed, no corners.
let playStoreGlyphRatio: CGFloat = 0.60
let playStorePixels = 512

/// (density folder, scale in half-steps). Android's densities are mdpi 1×,
/// hdpi 1.5×, xhdpi 2×, xxhdpi 3×, xxxhdpi 4×; storing them as halves (2, 3, 4,
/// 6, 8) keeps every size an integer: the foreground canvas is 108 dp
/// (108 × scale / 2 px) and the legacy icons are 48 dp (48 × scale / 2 px).
let densities: [(folder: String, scale: Int)] = [
    ("mdpi", 2), ("hdpi", 3), ("xhdpi", 4), ("xxhdpi", 6), ("xxxhdpi", 8),
]
func foregroundPixels(_ scale: Int) -> Int { 108 * scale / 2 }
func legacyPixels(_ scale: Int) -> Int { 48 * scale / 2 }

// MARK: - Paths

let repositoryRoot = URL(fileURLWithPath: #filePath)
    .deletingLastPathComponent()   // tools/hakfa/
    .deletingLastPathComponent()   // tools/
    .deletingLastPathComponent()   // repository root
let mainRoot = repositoryRoot.appendingPathComponent("android/app/src/main")
func resURL(_ density: String, _ name: String) -> URL {
    mainRoot.appendingPathComponent("res/mipmap-\(density)/\(name).webp")
}
let playStoreURL = mainRoot.appendingPathComponent("ic_launcher-playstore.png")
let isCheckOnly = CommandLine.arguments.contains("--check")

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data("error: \(message)\n".utf8))
    exit(1)
}

// MARK: - The glyph

func glyphOutline() -> CGPath {
    guard let font = NSFont(name: fontPostScriptName, size: 100) else {
        fail("font '\(fontPostScriptName)' is not installed")
    }
    guard font.fontName == fontPostScriptName else {
        fail("font '\(fontPostScriptName)' resolved to '\(font.fontName)'")
    }
    let ctFont = font as CTFont
    let line = CTLineCreateWithAttributedString(NSAttributedString(string: glyph, attributes: [.font: font]))
    let outline = CGMutablePath()
    var count = 0
    for run in CTLineGetGlyphRuns(line) as! [CTRun] {
        let runFont = (CTRunGetAttributes(run) as NSDictionary)[kCTFontAttributeName] as! CTFont
        guard CTFontCopyPostScriptName(runFont) as String == fontPostScriptName else {
            fail("'\(glyph)' fell back to \(CTFontCopyPostScriptName(runFont))")
        }
        let n = CTRunGetGlyphCount(run)
        var glyphs = [CGGlyph](repeating: 0, count: n)
        var positions = [CGPoint](repeating: .zero, count: n)
        CTRunGetGlyphs(run, CFRange(location: 0, length: n), &glyphs)
        CTRunGetPositions(run, CFRange(location: 0, length: n), &positions)
        for (g, p) in zip(glyphs, positions) {
            guard let path = CTFontCreatePathForGlyph(ctFont, g, nil) else { fail("no outline for '\(glyph)'") }
            outline.addPath(path, transform: CGAffineTransform(translationX: p.x, y: p.y))
            count += 1
        }
    }
    guard count == glyph.count else { fail("'\(glyph)' laid out as \(count) glyphs") }
    // Same serialisation as make-app-icon.swift, so the fingerprint is comparable.
    var text = ""
    outline.applyWithBlock { element in
        let e = element.pointee
        let n: Int
        switch e.type {
        case .moveToPoint, .addLineToPoint: n = 1
        case .addQuadCurveToPoint: n = 2
        case .addCurveToPoint: n = 3
        default: n = 0
        }
        text += "\(e.type.rawValue)"
        for point in UnsafeBufferPointer(start: e.points, count: n) {
            text += String(format: " %.6f %.6f", point.x, point.y)
        }
        text += "\n"
    }
    let found = SHA256.hash(data: Data(text.utf8)).map { String(format: "%02x", $0) }.joined()
    guard found == approvedFingerprint else {
        fail("""
            '\(glyph)' in \(fontPostScriptName) no longer matches the approved outline.
              approved: \(approvedFingerprint)
              this Mac: \(found)
            """)
    }
    return outline
}

// MARK: - Rasterising

enum Shape { case transparent, roundedSquare, circle, square }

/// One square page: optional backdrop `shape` in the tile colour, then the glyph's
/// longer side at `glyphRatio` of the page, centred on its own ink.
func render(pixels: Int, shape: Shape, glyphRatio: CGFloat, outline: CGPath) -> CGImage {
    guard let colorSpace = CGColorSpace(name: CGColorSpace.sRGB),
          let context = CGContext(
            data: nil, width: pixels, height: pixels, bitsPerComponent: 8, bytesPerRow: 0,
            space: colorSpace, bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)
    else { fail("could not allocate a \(pixels)×\(pixels) bitmap") }
    let side = CGFloat(pixels)
    let page = CGRect(x: 0, y: 0, width: side, height: side)
    switch shape {
    case .transparent: break
    case .square: context.addRect(page)
    case .circle: context.addEllipse(in: page)
    case .roundedSquare:
        context.addPath(CGPath(roundedRect: page, cornerWidth: side * legacySquareCornerRatio,
                               cornerHeight: side * legacySquareCornerRatio, transform: nil))
    }
    if shape != .transparent {
        context.setFillColor(red: tileColor.red, green: tileColor.green, blue: tileColor.blue, alpha: 1)
        context.fillPath()
    }
    let bounds = outline.boundingBox
    let target = side * glyphRatio
    let scale = target / max(bounds.width, bounds.height)
    var scaling = CGAffineTransform(scaleX: scale, y: scale)
    guard let scaled = outline.copy(using: &scaling) else { fail("could not scale the outline") }
    let sb = scaled.boundingBox
    var centring = CGAffineTransform(translationX: (side - sb.width) / 2 - sb.minX,
                                     y: (side - sb.height) / 2 - sb.minY)
    guard let placed = scaled.copy(using: &centring) else { fail("could not centre the outline") }
    context.setFillColor(red: inkColor.red, green: inkColor.green, blue: inkColor.blue, alpha: 1)
    context.addPath(placed)
    context.fillPath()
    guard let image = context.makeImage() else { fail("could not read back the page") }
    return image
}

func pngData(_ image: CGImage) -> Data {
    let data = NSMutableData()
    guard let dest = CGImageDestinationCreateWithData(data, "public.png" as CFString, 1, nil) else {
        fail("could not open a PNG encoder")
    }
    CGImageDestinationAddImage(dest, image, nil)
    guard CGImageDestinationFinalize(dest) else { fail("could not encode a PNG") }
    return data as Data
}

@discardableResult
func run(_ launchPath: String, _ arguments: [String]) -> Int32 {
    let process = Process()
    process.executableURL = URL(fileURLWithPath: launchPath)
    process.arguments = arguments
    do { try process.run() } catch { fail("could not run \(launchPath): \(error)") }
    process.waitUntilExit()
    return process.terminationStatus
}

func which(_ tool: String) -> String? {
    for dir in ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"] {
        let path = "\(dir)/\(tool)"
        if FileManager.default.isExecutableFile(atPath: path) { return path }
    }
    return nil
}

// MARK: - Main

guard let cwebp = which("cwebp") else { fail("cwebp not found (brew install webp)") }
let staging = URL(fileURLWithPath: NSTemporaryDirectory())
    .appendingPathComponent("hakfa-android-icon-\(ProcessInfo.processInfo.processIdentifier)")
try? FileManager.default.createDirectory(at: staging, withIntermediateDirectories: true)
defer { try? FileManager.default.removeItem(at: staging) }

let outline = glyphOutline()
var outputs: [(staged: URL, committed: URL)] = []

/// Lossless WebP, like the files this replaces: pixels stay exactly as rasterised.
func stageWebp(_ image: CGImage, committed: URL, label: String) {
    let png = staging.appendingPathComponent("\(label).png")
    let webp = staging.appendingPathComponent("\(label).webp")
    do { try pngData(image).write(to: png) } catch { fail("could not write \(png.path)") }
    guard run(cwebp, ["-quiet", "-lossless", "-exact", "-z", "9", png.path, "-o", webp.path]) == 0 else {
        fail("cwebp failed on \(label)")
    }
    outputs.append((webp, committed))
}

for d in densities {
    let fg = foregroundPixels(d.scale)
    let legacy = legacyPixels(d.scale)
    let foreground = render(pixels: fg, shape: .transparent, glyphRatio: adaptiveGlyphRatio, outline: outline)
    if d.folder == "xxxhdpi" { checkSafeZone(foreground, pixelsPerDp: CGFloat(d.scale) / 2) }
    stageWebp(foreground, committed: resURL(d.folder, "ic_launcher_foreground"), label: "fg-\(d.folder)")
    stageWebp(render(pixels: legacy, shape: .roundedSquare, glyphRatio: legacySquareGlyphRatio, outline: outline),
              committed: resURL(d.folder, "ic_launcher"), label: "legacy-\(d.folder)")
    stageWebp(render(pixels: legacy, shape: .circle, glyphRatio: legacyRoundGlyphRatio, outline: outline),
              committed: resURL(d.folder, "ic_launcher_round"), label: "round-\(d.folder)")
}
let stagedPlay = staging.appendingPathComponent("playstore.png")
do {
    try pngData(render(pixels: playStorePixels, shape: .square, glyphRatio: playStoreGlyphRatio, outline: outline))
        .write(to: stagedPlay)
} catch { fail("could not write the Play icon") }
outputs.append((stagedPlay, playStoreURL))

/// The farthest ink of the adaptive foreground must stay inside the 66 dp safe
/// circle, or a circular launcher would clip 客.
func checkSafeZone(_ image: CGImage, pixelsPerDp: CGFloat) {
    guard let space = CGColorSpace(name: CGColorSpace.sRGB),
          let context = CGContext(data: nil, width: image.width, height: image.height, bitsPerComponent: 8,
                                  bytesPerRow: image.width * 4, space: space,
                                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)
    else { fail("could not read back the foreground") }
    context.draw(image, in: CGRect(x: 0, y: 0, width: image.width, height: image.height))
    guard let data = context.data else { fail("could not read back the foreground") }
    let bytes = data.assumingMemoryBound(to: UInt8.self)
    let centre = CGFloat(image.width) / 2
    var farthest: CGFloat = 0
    for y in 0..<image.height {
        for x in 0..<image.width where bytes[(y * image.width + x) * 4 + 3] > 40 {
            farthest = max(farthest, hypot(CGFloat(x) + 0.5 - centre, CGFloat(y) + 0.5 - centre))
        }
    }
    let dp = farthest / pixelsPerDp
    print(String(format: "safe zone: farthest ink %.1f dp of %.0f dp", dp, safeZoneRadiusDp))
    guard dp <= safeZoneRadiusDp else {
        fail(String(format: "the glyph reaches %.1f dp from the centre, outside the %.0f dp safe zone — lower adaptiveGlyphRatio", dp, safeZoneRadiusDp))
    }
}

/// RGBA pixels of an image file (PNG or WebP), for comparing without trusting encoder bytes.
func pixels(of url: URL) -> [UInt8]? {
    guard let source = CGImageSourceCreateWithURL(url as CFURL, nil),
          let image = CGImageSourceCreateImageAtIndex(source, 0, nil),
          let space = CGColorSpace(name: CGColorSpace.sRGB),
          let context = CGContext(data: nil, width: image.width, height: image.height, bitsPerComponent: 8,
                                  bytesPerRow: image.width * 4, space: space,
                                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)
    else { return nil }
    context.draw(image, in: CGRect(x: 0, y: 0, width: image.width, height: image.height))
    guard let data = context.data else { return nil }
    return Array(UnsafeBufferPointer(start: data.assumingMemoryBound(to: UInt8.self), count: image.width * image.height * 4))
}

var stale = 0
for output in outputs {
    let relative = output.committed.path.replacingOccurrences(of: repositoryRoot.path + "/", with: "")
    let same = FileManager.default.fileExists(atPath: output.committed.path)
        && pixels(of: output.staged) == pixels(of: output.committed)
        && pixels(of: output.staged) != nil
    if isCheckOnly {
        if !same { stale += 1; print("STALE  \(relative)") } else { print("ok     \(relative)") }
    } else {
        try? FileManager.default.createDirectory(at: output.committed.deletingLastPathComponent(), withIntermediateDirectories: true)
        try? FileManager.default.removeItem(at: output.committed)
        do { try FileManager.default.copyItem(at: output.staged, to: output.committed) }
        catch { fail("could not write \(relative): \(error)") }
        print(same ? "same   \(relative)" : "wrote  \(relative)")
    }
}
if isCheckOnly && stale > 0 { fail("\(stale) file(s) differ from what the script produces") }
