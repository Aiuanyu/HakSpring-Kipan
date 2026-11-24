import Foundation

/// Opt-in anonymous usage statistics for HakSpring-Kipan.
/// Sends only platform, app version, and selected lect — never IP, device ID, or typed text.
enum AnonymousTelemetry {
    private static let statsEndpoint = URL(string: "https://hakspring.pages.dev/api/kipan-stats")!

    private static func todayKey() -> String {
        let formatter = DateFormatter()
        formatter.calendar = Calendar(identifier: .gregorian)
        formatter.timeZone = TimeZone(identifier: "Asia/Taipei")
        formatter.dateFormat = "yyyy-MM-dd"
        return formatter.string(from: Date())
    }

    @MainActor
    static func reportIfOptedIn(force: Bool = false) async {
        let defaults = UserDefaults.standard
        guard defaults.bool(forKey: SettingsStore.Keys.anonymousTelemetryEnabled.name) else { return }

        let today = todayKey()
        let dateKey = SettingsStore.Keys.lastAnonymousTelemetryDate.name
        if !force && defaults.string(forKey: dateKey) == today {
            return
        }

        let rawLect = defaults.string(forKey: SettingsStore.Keys.lect.name)
            ?? SettingsStore.Keys.lect.defaultValue.rawValue
        let body: [String: Any] = [
            "event_type": "optin_stats",
            "platform": "macos",
            "version": AppVersion.installed,
            "dialect": rawLect,
            "payload": [
                "source": "macos_app",
            ],
        ]
        guard let jsonData = try? JSONSerialization.data(withJSONObject: body) else { return }

        var request = URLRequest(url: statsEndpoint)
        request.httpMethod = "POST"
        request.setValue("application/json; charset=utf-8", forHTTPHeaderField: "Content-Type")
        request.timeoutInterval = 10
        request.httpBody = jsonData

        do {
            let (_, response) = try await URLSession.shared.data(for: request)
            if let http = response as? HTTPURLResponse, (200 ..< 300).contains(http.statusCode) {
                defaults.set(today, forKey: dateKey)
            }
        } catch {
            // Silently ignore network errors; never disturb typing.
        }
    }
}
