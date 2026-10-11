// Keyboard-dismissal cleanup: lets a dismissed controller drop its state and be released (also run from deinit).

import Foundation
import KeyboardKit
import UIKit

// MARK: - Cleanup Operations

extension KeyboardViewController {
    // Idempotent entry point — clears input state, service references, the emoji delegate,
    // subscriptions and the keyboard view's hosting controller.
    //
    // A re-shown keyboard gets a NEW controller (iOS 27, simulator and device), and UIKit may
    // keep the dismissed one until its host app returns; after this it must neither react to
    // settings changes nor call into the process-shared engine.
    func performCleanup() {
        guard !isCleanedUp else {
            return
        }

        isCleanedUp = true
        cancellables.removeAll()

        cleanupInputState()
        cleanupServices()

        if let emojiService = emojiServiceStorage {
            emojiService.delegate = nil
            emojiServiceStorage = nil
        }

        detachKeyboardHostingController()
    }

    /// Unparents KeyboardKit's hosting controller; the view stays so the dismissal still shows
    /// keys. Must run in `viewWillDisappear`: a hosting controller still parented when its view
    /// leaves the window leaves SwiftUI's focus bridge holding this controller (one leaked
    /// controller per re-show), and unparenting in `viewDidDisappear` is already too late.
    func detachKeyboardHostingController() {
        for child in children {
            child.willMove(toParent: nil)
            child.removeFromParent()
        }
    }

    func cleanupServices() {
        actionHandler = nil
    }

    /// Ensure clean state for next keyboard activation
    func cleanupInputState() {
        if let handler = actionHandler {
            handler.composingManager.reset()
            handler.hostText.clear()
            handler.nextWordController.tearDown()
        }
        state.autocompleteContext.reset()
    }
}
