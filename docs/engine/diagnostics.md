# Diagnostic Service

> **Type**: Feature
> **Keywords**: `Diagnostics`, `DeviceInfo`, `BugReport`
> **Related**: ../ui/app-ui.md

---

## Summary

- Collects minimal device/app metadata for bug reporting
- User-initiated only — no automatic collection or transmission
- Three sharing methods: Copy, Share, Email

---

## Data Model

| Field | Type | Description |
|-------|------|-------------|
| `appVersion` | String | App version |
| `buildNumber` | String (iOS) / Int (Android) | Build number |
| `osVersion` | String | OS version |
| `deviceModel` | String | Hardware model identifier |

**Formatted output** (`DiagnosticInfo.formatted()`):
```
App: v<version> (<build>)
OS: iOS 17.3.1                      (Android: "OS: Android <release> (API <SDK_INT>)")
Device: iPhone14,5
```

---

## Collection Sources

| Field | iOS | Android |
|-------|-----|---------|
| appVersion | `CFBundleShortVersionString` | `PackageManager` `versionName` |
| buildNumber | `CFBundleVersion` | `BuildConfig.VERSION_CODE` |
| osVersion | `UIDevice.current.systemVersion` | `Build.VERSION.RELEASE` (+ `SDK_INT` in the formatted text) |
| deviceModel | `utsname()` syscall | `Build.MANUFACTURER` + `Build.MODEL` |

---

## Data Flow

```
User taps button (Copy/Share/Email)
  → DiagnosticService.gather()
  → DiagnosticInfo
  → .formatted() → plain text
  → Platform sharing mechanism
    ├─ Copy: UIPasteboard / ClipboardManager
    ├─ Share: ShareLink / Intent.ACTION_SEND
    └─ Email: mailto URL → info@taigikeyboard.tw
```

---

## User Interface

Located in the **Settings** tab, "Device Info" section.

| Action | Label | Mechanism |
|--------|-------|-----------|
| Copy | "Copy Device Info" | Pasteboard/Clipboard, with visual feedback |
| Share | "Share Device Info" | Native share sheet |
| Email | "Email a Bug Report" | Pre-filled mailto with subject "TaigiKeyboard Bug Report (v{version})" |

---

## Privacy

- **No data transmission** — user controls when and how to share
- **User-initiated only** — no background collection or analytics
- **Minimal scope** — only device/OS/app metadata; no input history, settings, dictionary, or user identification
- **No special permissions** required on either platform

---

## Platform Correspondence

| Function | iOS (`App/Tabs/Settings/`) | Android (`ui/tabs/settings/`) |
|----------|-----|---------|
| Service | `DiagnosticService.swift` (enum, static `gather()`) | `DiagnosticService.kt` (object) |
| UI | `SettingsTab.swift`, the Device Info `Section` | `InputSettingsScreen.kt` `DiagnosticSection`, fed by `DiagnosticViewModel.kt` |
| Strings | `i18n/settings.json` `diagnostic*` keys | same |
