// Hakfa fork: when the lect flash is owed.

@testable import TaigiInputMethodCore
import XCTest

final class InputSourceArrivalTests: XCTestCase {
    /// The process starts because the user selected this input method: that first activation is
    /// an arrival, and it is owed once.
    func testLaunchingAsTheSelectedSource_owesOneFlash() {
        var tracker = InputSourceArrivalTracker(isOursAtLaunch: true)
        XCTAssertTrue(tracker.consumePending())
        XCTAssertFalse(tracker.consumePending(), "once per arrival")
    }

    func testLaunchingWhileAnotherSourceIsSelected_owesNothing() {
        var tracker = InputSourceArrivalTracker(isOursAtLaunch: false)
        XCTAssertFalse(tracker.consumePending())
    }

    /// Switching from another input method to this one is an arrival.
    func testSelectingThisSourceFromAnother_owesAFlash() {
        var tracker = InputSourceArrivalTracker(isOursAtLaunch: false)
        tracker.selectionChanged(isOurs: true)
        XCTAssertTrue(tracker.isFlashPending)
        XCTAssertTrue(tracker.consumePending())
    }

    /// The system reports the selection again with this source still selected (and IMK
    /// re-activates the session on every focus change): neither is an arrival.
    func testStayingOnThisSource_owesNothingMore() {
        var tracker = InputSourceArrivalTracker(isOursAtLaunch: true)
        XCTAssertTrue(tracker.consumePending())
        tracker.selectionChanged(isOurs: true)
        XCTAssertFalse(tracker.consumePending())
    }

    func testLeavingAndComingBack_owesAFlashEachTime() {
        var tracker = InputSourceArrivalTracker(isOursAtLaunch: true)
        XCTAssertTrue(tracker.consumePending())
        tracker.selectionChanged(isOurs: false)
        XCTAssertFalse(tracker.isFlashPending, "leaving owes nothing")
        tracker.selectionChanged(isOurs: true)
        XCTAssertTrue(tracker.consumePending())
    }

    /// An arrival nobody showed yet is not lost when the user hops away again before a session
    /// activates: it stays owed until consumed.
    func testAnUnshownArrival_staysOwedUntilConsumed() {
        var tracker = InputSourceArrivalTracker(isOursAtLaunch: false)
        tracker.selectionChanged(isOurs: true)
        tracker.selectionChanged(isOurs: false)
        XCTAssertTrue(tracker.isFlashPending)
    }
}
