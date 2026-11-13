// Hakfa fork: metadata about bundled dictionary assets from `data_version.json`.

import Foundation

struct LexiconVersionInfo: Sendable {
    let version: String
    let builtAt: String
    let recordCount: Int

    var displaySummary: String {
        if recordCount > 0 {
            let formatter = NumberFormatter()
            formatter.numberStyle = .decimal
            let formattedCount = formatter.string(from: NSNumber(value: recordCount)) ?? "\(recordCount)"
            return "\(version)（\(formattedCount) 條詞）"
        }
        return version
    }
}

enum LexiconVersion {
    private struct Wire: Decodable {
        let hakspringDataVersion: String?
        let builtAt: String?
        let recordCount: Int?

        enum CodingKeys: String, CodingKey {
            case hakspringDataVersion = "hakspring_data_version"
            case builtAt = "built_at"
            case recordCount = "record_count"
        }
    }

    static let current: LexiconVersionInfo = load()

    private static func load() -> LexiconVersionInfo {
        let candidateURLs = [
            Bundle.main.url(forResource: "data_version", withExtension: "json"),
            Bundle.main.resourceURL?.appendingPathComponent("data_version.json"),
            URL(fileURLWithPath: "assets/dictionaries/data_version.json"),
            URL(fileURLWithPath: "../assets/dictionaries/data_version.json"),
            URL(fileURLWithPath: "../../assets/dictionaries/data_version.json"),
            URL(fileURLWithPath: "../../../assets/dictionaries/data_version.json"),
        ].compactMap { $0 }

        for url in candidateURLs {
            if let data = try? Data(contentsOf: url),
               let wire = try? JSONDecoder().decode(Wire.self, from: data) {
                return LexiconVersionInfo(
                    version: wire.hakspringDataVersion ?? "unknown",
                    builtAt: wire.builtAt ?? "",
                    recordCount: wire.recordCount ?? 0
                )
            }
        }
        return LexiconVersionInfo(version: "unknown", builtAt: "", recordCount: 0)
    }
}
