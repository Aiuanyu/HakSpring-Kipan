// Hakfa fork: tell the user which lect they are typing in when they arrive from another input
// method (the mode flash, once), not on every focus change.

import AppKit
import Carbon.HIToolbox

/// Decides when the lect flash is owed. Pure, so the rule is testable without the system's
/// input-source machinery.
///
/// A flash is owed when the selected input source BECOMES ours (it was not before), and once
/// at launch (the system starts this process because the user selected it, so the first
/// activation is an arrival). Focus changes between apps or fields while ours stays selected
/// are not arrivals: IMK deactivates and activates the session for those too, which is why the
/// flash is not hung on `activateServer` alone.
struct InputSourceArrivalTracker: Equatable {
    /// Whether the last observed selected input source was ours.
    private(set) var wasOurs: Bool
    /// An arrival not yet shown: the session that should show it may not be active yet when
    /// the notification lands (IMK activates the client and the system posts the change in no
    /// fixed order), so it waits here for whichever side looks first.
    private(set) var isFlashPending: Bool

    /// - Parameter isOursAtLaunch: the selected source when the process starts.
    init(isOursAtLaunch: Bool) {
        wasOurs = isOursAtLaunch
        isFlashPending = isOursAtLaunch
    }

    /// The system reported a selection change.
    mutating func selectionChanged(isOurs: Bool) {
        if isOurs, !wasOurs {
            isFlashPending = true
        }
        wasOurs = isOurs
    }

    /// A session asks whether to flash now; true at most once per arrival.
    mutating func consumePending() -> Bool {
        defer { isFlashPending = false }
        return isFlashPending
    }
}

/// Watches the selected input source for the life of the process.
@MainActor
enum InputSourceArrival {
    private static var tracker = InputSourceArrivalTracker(isOursAtLaunch: false)
    private static var isInstalled = false

    private static let notificationName = Notification.Name(
        "com.apple.Carbon.TISNotifySelectedKeyboardInputSourceChanged",
    )

    /// Called once, at launch (`AppDelegate`).
    static func install() {
        guard !isInstalled else { return }
        isInstalled = true
        tracker = InputSourceArrivalTracker(isOursAtLaunch: selectedSourceIsOurs())
        DistributedNotificationCenter.default().addObserver(
            forName: notificationName, object: nil, queue: .main,
        ) { _ in
            MainActor.assumeIsolated {
                tracker.selectionChanged(isOurs: selectedSourceIsOurs())
                // The session may already be active; if not, `activateServer` finds the flag.
                // A turn later so a session activating in the same breath wins the race
                // cleanly instead of both flashing.
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) {
                    MainActor.assumeIsolated { flashThroughTheFocusedSessionIfOwed() }
                }
            }
        }
    }

    /// `activateServer` asks: is the flash owed to this session?
    static func consumePending() -> Bool {
        tracker.consumePending()
    }

    private static func flashThroughTheFocusedSessionIfOwed() {
        guard tracker.isFlashPending else { return }
        if ComposingSessionCoordinator.shared.flashLectOnArrival() {
            _ = tracker.consumePending()
        }
    }

    /// Whether the system's selected keyboard input source is this input method's.
    private static func selectedSourceIsOurs() -> Bool {
        guard
            let bundleID = Bundle.main.bundleIdentifier,
            let source = TISCopyCurrentKeyboardInputSource()?.takeRetainedValue(),
            let raw = TISGetInputSourceProperty(source, kTISPropertyInputSourceID)
        else { return false }
        let id = Unmanaged<CFString>.fromOpaque(raw).takeUnretainedValue() as String
        return id.hasPrefix(bundleID)
    }
}
