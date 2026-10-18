# HakSpring character supplement (客源翠補字典)

The MOE Hakfa dictionary (教典) and the certification word lists (認證) have no single-character entry for many common characters (Sixian: 藍, 理, 吔, 公, 運 …), so they could only be typed inside a word. HakSpring derives their readings from the words they appear in and ships them as a separate data source; Kipan loads it as its own dictionary source.

## Data

- Produced in the HakSpring repo: `data/supp/補字典-{四,海,大,平,安,南}.json`, by `python3 process_all_data.py --only-supp` (`scripts/build_supp_dict.py`). See HakSpring `data/supp/README.md`.
- Rule: a word whose character count equals its syllable count is aligned character by character. Two kinds of rows, told apart by the `分類` column:
    - `補字典`: the character has no single-character entry in that lect; every reading found in words is kept.
    - `多音字補音` (polyphone readings): the character has a single-character entry, but a reading found in words is missing from it (e.g. 發 has only `bod`; `fad` occurs 65 times in words). Kept from 3 occurrences on. Readings that differ only in tone are skipped by Kipan's `word|toneless` key, since input is toneless anyway.
- Kipan currently reads only `補字典-四.json` (Sixian), in `dictionary/sources/official/hakspring/extract_and_build.js` step 3.

## Source bit

| | Value |
|---|---|
| Bitmask | bit 6 (`1 << 6`), upstream internal name `kungge` (soft fork: internal names unchanged) |
| `kautian_subtag` | `0` (not a kautian row) |
| Frequency | `min(9000, 1000 + 100 × occurrences)` — always below real dictionary words (≥ 19,000) |
| `hakspring.csv` | new last column `bitmask` |

Bit 6 follows the Obsidian `dialects` plan table (教典 0, 認證 1, … 客源翠補字典 6).

## Locked on

The source cannot be turned off, by design:

| Layer | Where |
|---|---|
| Engine settings (macOS, Windows, Linux) | `desktop/crates/taigi-desktop-core/src/settings/document.rs` `dictionary_sources()` returns `kungge: true` regardless of the stored value |
| Engine settings (Android) | `PrefHelper.isKunggeDictEnabled` returns `true` |
| macOS UI | `DictionaryTogglesView.swift`: first row of the supplement section, `.constant(true)` + `.disabled(true)` |
| Android UI | `DictionarySourcesScreen.kt`: first row of the supplement card, `enabled = false`; `DictionaryRowWithDescription` gained an `enabled` parameter |
| Label | `i18n/hakfa/common.json` `kunggeDict`, `i18n/hakfa/dictionary.json` `kunggeDescription` |

iOS, Windows and Linux settings windows still show the upstream toggle; the engine side is already locked for Windows/Linux.

## Tests

- `desktop/crates/taigi-desktop-core/tests/engine_roundtrip.rs`: `hakfa_supp_dict_single_chars_reachable` (`lam`→藍, `li`→理, `gung`→公, `iun`→運; polyphone `fad`→發, `dan`→單), `hakfa_supp_dict_is_its_own_source` (bit 6 off at the engine → 藍 gone).
- `settings/document.rs`: `hakfa_supp_dict_source_is_locked_on`.

## Rebuilding

When HakSpring's certification or MOE data changes: in HakSpring run `python3 process_all_data.py` (it rebuilds the supplement last), then in Kipan `node dictionary/sources/official/hakspring/extract_and_build.js`.

Set `HAKSPRING_ROOT` to build from another HakSpring checkout, e.g. a worktree whose data is not on `main` yet: `HAKSPRING_ROOT=../HakSpring-supp node dictionary/sources/official/hakspring/extract_and_build.js`.
