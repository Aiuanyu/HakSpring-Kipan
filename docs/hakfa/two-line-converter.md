# In-Place Two-Line Converter (Dual-Script Annotation to Parallel Lines)

> Public user documentation & full-width/half-width punctuation table: [HakSpring Dual-Scripter Guide](https://hakspring.pages.dev/page.html?p=dual-scripter) ([`pages/dual-scripter.md`](https://github.com/Aiuanyu/HakSpring/blob/main/pages/dual-scripter.md)).

HakSpring Kipan can convert inline dual-script annotations directly in the active text editor into two clean parallel lines (Hanji on one line, romanization on the other).

## Supported Inline Formats

- Round brackets: `𠊎 (Ngǎi) 識 (sii) 講 (gòng) 客 (Hàg)。` and full-width `客源翠（Hàg-Ngiǎn-Cui）`
- Square brackets: `無成功 [mǒ sǐin'gúng] 無反應 [mǒ fàn'in]` and full-width `［...］`
- Curly brackets: `試驗 {Cii-ngiàm} 好 {hò}` and full-width `｛...｝`
- HTML Ruby tags: `<ruby>𠊎<rt>Ngǎi</rt></ruby>` and `<rt>` shorthand

## Conversion Rules

1. **Top-line order detection**: If Hanji appears outside the brackets first, Hanji is placed on the top line (`han-top`); if romanization appears outside the brackets first (`Ngǎi (𠊎)`), romanization is placed on the top line (`rom-top`).
2. **Bidirectional punctuation**:
   - Hanji line keeps/restores full-width punctuation (`，`, `。`, `、`, `！`, `？`, `……`, etc.).
   - Romanization line converts CJK punctuation to half-width equivalents (including `、` → `, ` and `……` → `...`) with proper spacing.
3. **Neutral tokens**: Unannotated ASCII tokens, numbers, and version strings (e.g., `v0.0.5`) embedded in a sentence are preserved on both lines.
4. **Multi-line input**: Newlines inside the input are preserved within each track, with a single `\n` separating the top block and bottom block (`topText + "\n" + bottomText`).

## Platform Implementations

### macOS (`⌃⌘T`)

- **Core**: `macos/Sources/TaigiInputMethodCore/Lexicon/KipanConverter.swift`
- **Trigger**: Global shortcut `convertKipanTwoLines` (default `⌃⌘T`), intercepted in `TaigiInputController.handle`.
- **Pipeline**:
  1. Reads the highlighted selection via `IMKTextInput.selectedRange()` and `attributedSubstring(forProposedRange:)` and replaces it in-place with `insertText(_:replacementRange:)`.
  2. Falls back to the system clipboard when the host app does not expose `selectedRange`.
  3. Shows a lect-aware HUD (`既經轉雙行` / `已經轉雙行`).

### Android (Smartbar Toolbar Button)

- **Core**: `android/app/src/main/java/com/siansiansu/taigikeyboard/ime/text/smartbar/HakfaKipanConverter.kt`
- **Trigger**: Toolbar button `toolbar_kipan_two_lines_button` (`@drawable/ic_kipan_two_lines`) in `smartbar.xml` (open via `+` on the Smartbar), wired through `ToolbarManager` and `SmartbarManager.convertKipanTwoLinesInPlace()`.
- **Why Smartbar + `InputConnection` instead of `ACTION_PROCESS_TEXT`**: Android's floating text-selection menu (`ACTION_PROCESS_TEXT`) varies across apps and OEM skins and often hides third-party actions in the `⋮` overflow menu. Running the conversion inside the active `InputMethodService` via `currentInputConnection` works uniformly across editors without requiring text selection first.
- **Pipeline**:
  1. Calls `finishComposingText()` inside a batch edit.
  2. If text is selected (`getSelectedText(0)`), converts and replaces the selection via `commitText`.
  3. Otherwise inspects `getTextBeforeCursor(4000, 0)` via `HakfaKipanConverter.extractConvertibleTarget()`, preferring the current paragraph (after the last `\n`) and falling back to the full lookbehind, then replaces it via `deleteSurroundingText` + `commitText`.
  4. If the editor exposes neither selection nor surrounding text, falls back to `ClipboardManager` and commits the converted result at the cursor.
  5. Auto-collapses the toolbar if `isToolbarAutoCollapse` is enabled and shows a Toast confirmation.

## Tests

- macOS: `macos/Tests/TaigiInputMethodCoreTests/KipanConverterTests.swift`
- Android: `android/app/src/test/java/com/siansiansu/taigikeyboard/ime/text/smartbar/HakfaKipanConverterTest.kt`
