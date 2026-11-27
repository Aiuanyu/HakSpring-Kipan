// KipanConverter.swift
// 客源翠企盤（HakSpring-Kipan）輸入法並列格式轉換器
//
// Thin Swift wrapper delegating to the shared Rust `two-line` engine
// via `RustEngineBridge` (protobuf over swift-bridge).

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

    public struct ConversionResult: Equatable, Sendable {
        public let topText: String
        public let bottomText: String
        public let combinedText: String
        public let topLines: [String]
        public let bottomLines: [String]
        public let pairs: [Pair]
        public let detectedOrder: String

        public init(
            topText: String,
            bottomText: String,
            combinedText: String,
            topLines: [String],
            bottomLines: [String],
            pairs: [Pair],
            detectedOrder: String
        ) {
            self.topText = topText
            self.bottomText = bottomText
            self.combinedText = combinedText
            self.topLines = topLines
            self.bottomLines = bottomLines
            self.pairs = pairs
            self.detectedOrder = detectedOrder
        }
    }

    /// 中文全形標點轉羅馬字半形標點
    public static func convertPunctToRomance(_ p: String) -> String {
        if p.isEmpty { return "" }
        return RustEngineBridge.convertPunctToRomance(p)
    }

    /// 羅馬字半形標點轉中文全形標點
    public static func convertPunctToHan(_ p: String) -> String {
        if p.isEmpty { return "" }
        return RustEngineBridge.convertPunctToHan(p)
    }

    /// 轉換整段企盤文字（支援多行段落）
    public static func convertKipanText(
        _ text: String,
        autoSpaceRomance: Bool = false,
        forceOrder: String = "auto"
    ) -> ConversionResult {
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
        guard let res = RustEngineBridge.convertKipanTwoLines(
            text: text,
            autoSpaceRomance: autoSpaceRomance,
            forceOrder: forceOrder
        ) else {
            return ConversionResult(
                topText: text,
                bottomText: text,
                combinedText: "\(text)\n\(text)",
                topLines: text.components(separatedBy: .newlines),
                bottomLines: text.components(separatedBy: .newlines),
                pairs: [],
                detectedOrder: "han-top"
            )
        }
        return ConversionResult(
            topText: res.topText,
            bottomText: res.bottomText,
            combinedText: res.combinedText,
            topLines: res.topLines,
            bottomLines: res.bottomLines,
            pairs: res.pairs.map { Pair(han: $0.han, rom: $0.rom) },
            detectedOrder: res.detectedOrder
        )
    }
}
