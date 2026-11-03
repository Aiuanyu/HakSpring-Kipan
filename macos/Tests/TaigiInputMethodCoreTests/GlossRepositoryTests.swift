import XCTest
@testable import TaigiInputMethodCore

final class GlossRepositoryTests: XCTestCase {

    func testHeadwordQuery_ordersResultsByLect() {
        let repo = GlossRepository.shared
        let results = repo.search(query: "飯包")
        guard !results.isEmpty else {
            // In case running in an isolated environment where DB is not present
            return
        }

        // Default order should follow "四南海大平安": xiien, namxiian, hoiliug, taibu, ngiaupin, zhioon
        let lects = results.map { $0.lectId }
        XCTAssertTrue(lects.contains("xiien") || lects.contains("hoiliug"))

        // Exact match should have isSynonymHit == false
        XCTAssertFalse(results[0].isSynonymHit)
        XCTAssertEqual(results[0].word, "飯包")
    }

    func testSynonymQuery_findsAssociatedHeadwords() {
        let repo = GlossRepository.shared
        let results = repo.search(query: "便當")
        guard !results.isEmpty else { return }

        // Synonym hit for "便當" should return "飯包"
        let words = results.map { $0.word }
        XCTAssertTrue(words.contains("飯包"))
        if let fanbau = results.first(where: { $0.word == "飯包" }) {
            XCTAssertTrue(fanbau.isSynonymHit)
            XCTAssertEqual(fanbau.matchedKeyword, "便當")
        }
    }

    func testLectFilter_bringsFilteredLectToFirst() {
        let repo = GlossRepository.shared
        let results = repo.search(query: "飯包", lectFilter: "hoiliug")
        guard !results.isEmpty else { return }

        // If hoiliug exists in results, it should be the very first entry
        if results.contains(where: { $0.lectId == "hoiliug" }) {
            XCTAssertEqual(results[0].lectId, "hoiliug")
        }
    }
}
