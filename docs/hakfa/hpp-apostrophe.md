# HagPinPlus apostrophe (`'`) in the None syllable separator

Kipan's Syllable Separator **None** is HagPinPlus run-together writing: no space between syllables, and an apostrophe where the joined form would be misread (`í-ien` 醫院 → `í'ien`, not `íien`). Hyphen and Space are unchanged. Decided 2026-10-19 (Obs `創作/程式/HakSpring/IME/HagPinPlus`): no separate "apostrophe" option — None and the apostrophe go together.

## Rules

Same as HakSpring Romaine (`js/romanizer.js` `applyApostropheRules`). Between a syllable ending in P and one starting with N, write `'` when:

| # | P | N |
|---|---|---|
| 1 | `b d g m n` | a vowel |
| 2 | `a` | `i`, `u` |
| 3 | `e` | `u` |
| 4 | `i` | a vowel |
| 5 | `o` | `i` |
| 6 | `u` | `a e i` |
| 7 | a vowel | `ng` |
| 8 | `n` | `g` |

Letters are compared on their base letter (tone marks stripped, NFD combining marks skipped, lowercased). A non-Latin neighbour (Hanji, punctuation, space, `·`) gets nothing.

## Code

- `engine/phonetics/src/hpp.rs` — `hpp_boundary(prev, next) -> "'" | ""`, plus tests.
- Two upstream seams call it, each behind a `hakfa:` comment, only when the joiner is `""` (None) and the boundary is a single hyphen (or none, between nailed segments):
    - `engine/phonetics/src/api.rs` `syllable_joiner_display` — candidate `roman`, next-word predictions.
    - `engine/composing/src/api.rs` `push_boundary` — the preedit / commit built from nailed segments (compound joins and typed `-`).
- Identity keys (`display_text`, `canonical_tl`, learned readings) never pass through here.
- Upstream tests that used Taigi `tâi-uân` under None now expect `tâi'uân` (5 tests, `hakfa:` comments).
- i18n: `i18n/hakfa/settings.json` `syllableSeparatorInfo`.

## Verification (2026-10-20)

All rows of `hakspring.csv` whose reading is letters and hyphens only (22,606) were rendered through `syllable_joiner_display(_, "")` and compared with Romaine's `applyApostropheRules` run on the same reading: 22,599 identical. The 7 differences are Romaine misses, Kipan is right:

- 6 rows with syllabic `m̌` / `ň` (base letter + U+030C, no precomposed form): Romaine's `[bdgmn] ` class does not match `m` + combining mark, so `m̌-iǎng` became `m̌iǎng`.
- `van-sii-ǐ-i`: Romaine's global regex consumes the shared `ǐ`, so the second boundary is missed (`vansii'ǐi`); Kipan gives `vansii'ǐ'i`.

3,756 of 23,124 rows gain an apostrophe under None.

Data finding: one reading in `hakspring.csv` carries a stray `特` (`特ň-ià-déu`), MOE's annotation prefix not stripped.

## Later

- Typed `'` as a syllable boundary on input (like `-`): not done.
- Other lects (Hailu, Dapu, Raoping, Zhao'an `oo ee bb`): re-check the rules after multi-lect P1.
