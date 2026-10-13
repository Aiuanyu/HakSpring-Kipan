# Keyboard Layout

> **Type**: Feature
> **Keywords**: `Layout`, `KeyDef`, `LayoutConverter`, `CustomLayoutService`, `LayoutManager`
> **Related**: ../architecture/system-overview.md, ../contributing/cross-platform-alignment.md §2

---

## Where layouts live

| | iOS (`ios/Sources/TaigiKeyboard/Layout/`) | Android |
|---|---|---|
| Definitions | `TaigiLayouts.swift` — each layout is a full `[[KeyDef]]` | `android/app/src/main/assets/ime/text/<type>/<name>.json` |
| Selection / loading | `CustomLayoutService.swift` → `LayoutConverter.swift` | `ime/text/layout/LayoutManager.kt` (merges main + modifier + extension layers) |
| Key model | `KeyDef.swift` | `ime/text/key/KeyData.kt` |

Concept → file map for both platforms: [cross-platform-alignment.md §2](../contributing/cross-platform-alignment.md#2-where-each-concept-lives).

## Alphabetic layouts

| Layout | Notes |
|---|---|
| PhahTaigi | `!` `?` replace `q` `w`; `,` `.` replace `z` `x`; digit top row |
| QWERTY | TL and POJ variants |
| QWERTY English | used in English input mode |
| TPS (方音符號) | always used in TPS input mode, whatever layout is selected |
| MOE1 (教育部輸入法佈局1) | TL and POJ variants |
| MOE2 (教育部輸入法佈局2) | TL and POJ variants |

Non-alphabetic pages: **Numeric** (common punctuation, quotes, CJK brackets) and **Symbolic** (programming brackets, arrows, currency, math); they do not overlap.

## Globe key (iOS)

Each iOS layout has an `_iPhone` form and a derived `_withGlobe` form (`.globe` inserted at bottom-row index 1). `CustomLayoutService.needsGlobeKey`: English mode → globe on iPad and Touch ID iPhones (`DeviceConfiguration`); TPS mode → never; otherwise the user's globe-key toggle.

`ScreenSizeClass` (`DeviceConfiguration.swift`: < 375 pt compact, 375–413 regular, ≥ 414 large, iPad) only picks the per-device candidate font base in `CandidateTheme`.

## Full-width / half-width punctuation

Half-width by default; the full-width punctuation setting switches `。，？！` etc. iOS: a key declares both forms (`.char(half, fullWidth: full)`) and `LayoutConverter` picks one; TPS is always full-width. Android: separate `*_fullwidth` / `*_halfwidth` JSON files chosen by `LayoutManager`. Programming, currency and math symbols have no full-width form.
