// Phonetics slice of the engine bridge: the one check the macOS controller
// makes itself before a key reaches desktop-core — whether the key's
// character could swap with the armed auto space (`ComposingBackend.panel`).

import Foundation

extension RustEngineBridge {
    /// Whether `text` is a single punctuation character that attaches to the
    /// preceding word under auto-space (`guá ` + `?` → `guá? `) — the
    /// engine's set, the one every platform asks. False when the engine
    /// fails: no swap, the space stays where it is.
    static func isAttachingPunctuation(_ text: String) -> Bool {
        let op = "isAttachingPunctuation"
        var check = Taigi_Engine_IsAttachingPunctuation()
        check.text = text
        var request = Taigi_Engine_PhoneticsRequest()
        request.method = .isAttachingPunctuation(check)
        guard let payload = roundtrip(payload: .phonetics(request), op: op) else { return false }
        guard case let .phonetics(response) = payload, case let .boolResult(result)? = response.result else {
            recordFailure(op: op, message: "expected a phonetics bool result, got \(payload)")
            return false
        }
        return result.value
    }

    static func convertPunctToRomance(_ text: String) -> String {
        let op = "convertPunctToRomance"
        var req = Taigi_Engine_ConvertPunctToRomance()
        req.text = text
        var request = Taigi_Engine_PhoneticsRequest()
        request.method = .convertPunctToRomance(req)
        guard let payload = roundtrip(payload: .phonetics(request), op: op) else { return text }
        guard case let .phonetics(response) = payload, case let .stringResult(result)? = response.result else {
            recordFailure(op: op, message: "expected a phonetics string result, got \(payload)")
            return text
        }
        return result.output
    }

    static func convertPunctToHan(_ text: String) -> String {
        let op = "convertPunctToHan"
        var req = Taigi_Engine_ConvertPunctToHan()
        req.text = text
        var request = Taigi_Engine_PhoneticsRequest()
        request.method = .convertPunctToHan(req)
        guard let payload = roundtrip(payload: .phonetics(request), op: op) else { return text }
        guard case let .phonetics(response) = payload, case let .stringResult(result)? = response.result else {
            recordFailure(op: op, message: "expected a phonetics string result, got \(payload)")
            return text
        }
        return result.output
    }

    static func convertKipanTwoLines(
        text: String,
        autoSpaceRomance: Bool = false,
        forceOrder: String = "auto"
    ) -> Taigi_Engine_KipanTwoLinesResult? {
        let op = "convertKipanTwoLines"
        var req = Taigi_Engine_ConvertKipanTwoLines()
        req.text = text
        req.autoSpaceRomance = autoSpaceRomance
        req.forceOrder = forceOrder
        var request = Taigi_Engine_PhoneticsRequest()
        request.method = .convertKipanTwoLines(req)
        guard let payload = roundtrip(payload: .phonetics(request), op: op) else { return nil }
        guard case let .phonetics(response) = payload, case let .kipanTwoLinesResult(result)? = response.result else {
            recordFailure(op: op, message: "expected a kipanTwoLinesResult, got \(payload)")
            return nil
        }
        return result
    }
}
