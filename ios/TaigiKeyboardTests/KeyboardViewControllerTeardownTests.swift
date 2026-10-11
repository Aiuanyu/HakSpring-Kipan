@testable import TaigiKeyboard
import UIKit
import XCTest

/// Teardown must unparent KeyboardKit's hosting controller in `viewWillDisappear`
/// (`KeyboardViewController+Cleanup.swift`). Deallocation needs a real extension host
/// (simulator `heap` count), so only the detach is checked here.
final class KeyboardViewControllerTeardownTests: XCTestCase {
    func testKeyboardViewController_willDisappear_unparentsHostingControllerAndKeepsView() {
        let controller = KeyboardViewController()
        controller.loadViewIfNeeded()
        controller.viewWillAppear(false)
        XCTAssertEqual(controller.children.count, 1, "precondition: KeyboardKit installs one hosting controller by viewWillAppear")
        let hostingView = controller.children.first?.view

        controller.viewWillDisappear(false)

        XCTAssertTrue(controller.children.isEmpty, "hosting controller still parented after viewWillDisappear: \(controller.children)")
        XCTAssertTrue(
            hostingView?.superview === controller.view,
            "hosting view must stay in place so the dismissal still shows the keys",
        )
        XCTAssertEqual(hostingView?.isUserInteractionEnabled, false, "detached keys must not reach the torn-down ActionHandler")
    }
}
