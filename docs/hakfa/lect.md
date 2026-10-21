# Lects (腔) and dictionary sources

Kipan will type five Hakfa lects (Sixian, Hailu, Dapu, Raoping, Zhao'an), switched as exclusive modes, with sub-lect variants (e.g. Sixian north / south / Meinong) as toggles inside a lect. Dictionary sources (MOE, certification lists, HakSpring supplement, …) are a separate axis. Design and decisions live in the maintainer's Obsidian `dialects` note; this page is the code-bound part.

Status: P0 (data foundation) done for Sixian. Nothing filters by lect yet.

## Record layout in `dictionary.bin`

Two `u16` fields per record carry the axes. Upstream semantics are kept where Kipan has no reason to change them.

### `bitmask` — dictionary source

| Bit | Kipan source | Upstream internal name | Engine settings |
|---|---|---|---|
| 0 | MOE Hakfa dictionary (教典) | `kautian` | user toggle (to be locked on, together with bit 1, in the settings redesign) |
| 1 | Certification word lists (認證詞彙) | `taigitv` | **locked on** |
| 6 | HakSpring character supplement (客源翠補字典) | `kungge` | **locked on** ([supp-dict.md](supp-dict.md)) |

A word found in both MOE and the certification lists is one record with bits 0 and 1 set; it keeps the certification reading and frequency. Planned: 2 loanword handbook, 3 MOE appendix, 4 name readings, 5 Thiên-kông dictionary; 7–11 reserved. Bits 9, 10 and 12 have upstream engine semantics (khiin gate, dev, variant) — do not reuse them.

### `kautian_subtag` — subcollection, variant, lect

| Bits | Meaning |
|---|---|
| 0 | main reading (`KAUTIAN_SUBTAG_MAIN_BIT`); set on MOE and certification rows, `0` on supplement rows |
| 1–10 | upstream accent columns; Kipan will use 1, 2, 3 for the lect's variants (Sixian: north, south, Meinong; Raoping: Hsinchu, Zhuolan, Taoyuan) — not written yet |
| 11 | upstream name appendix — not used |
| 12 | upstream alternative reading (又唸作) — candidate for secondary readings split by `splitReadingVariants()`, not written yet |
| **13–15** | **lect code** (Kipan) |

Lect codes: `1` Sixian (四). Reserved: `2` Hailu, `3` Dapu, `4` Raoping, `5` Zhao'an. `0` means "no lect" and fails the build.

The engine reader masks the subtag with `KAUTIAN_SUBTAG_USED_MASK` (bits 0–12), so the lect code is invisible to filtering today. Reading it requires widening that mask (upstream line in `engine/lexicon/src/dictionary_reader.rs`; keep the change small and commented).

The lect is not passed to the engine inside `enabled_sources_bitmask` (only bits 27–31 are free there); it will get its own settings field.

## Ranking

`engine/ranking/src/score.rs` `CONTINUOUS_SOURCE_BITS`: Kipan ranks bit 1 (certification) at `1`, the same as bit 0 (MOE); upstream ranks `taigitv` at `2`. Source rank is only the last tiebreak (after walker cost, score and frequency), but with bit 1 at rank 2 a certification-only word lost every frequency tie to a word also in MOE (`ban`: 半 → 搬, `dai`: 帶 → 低). Equal rank keeps dictionary order, i.e. the candidates from before the source split. Certification already outranks MOE through frequency (30,000–50,000 vs. ≤ 20,000).

## Build

```bash
node dictionary/sources/official/hakspring/extract_and_build.js [--lect 四]
```

- `--lect` defaults to `四`; any other value fails until its tone-value table is added (Hailu MOE readings are tone numbers; HakSpring `tone_mapping.json` has the table).
- Inputs per lect: `data/cert/113<lect><level>.json`, `data/gip/20250630-<lect>.json`, `data/supp/補字典-<lect>.json` under `HAKSPRING_ROOT`.
- Build assertions: every row has a known source bit and a lect code; every MOE row has a subcollection bit (else the engine drops it once subcollection filtering is active).
- `hakspring.csv` gained a `lect` column.

## Reading notes in the source data

`splitReadingVariants` in `extract_and_build.js` splits a reading field on the notes HakSpring lists (`data/annotations.md` §3, `scripts/build_supp_dict.py` `READING_NOTES` / `DROP_NOTES`, kept in step):

- Another reading, kept: 又俗音 又讀 又音 俗音 特殊音 特 又 文 白 (`特diǒng` → `diǒng`).
- A MOE 特 / 特殊音 reading keeps its own row even when it differs from another reading only in tone (骨髓 `gùd-súi` and 特 `gùd-sǔi`; maintainer 2026-10-21). Every other reading is keyed by headword + toneless spelling, so readings that differ only in tone collapse into one row; the certification 【】 variants among them wait for the variant bits (Obs `dialects` Phase 1).
- A contraction or a sandhi form, dropped: 合音(讀), 小稱變調讀(本調為), (為)後字變調(讀), 本調為 (`lǒi-hi 合音lí 合音lói` → `lǒi-hi`).

`/` and parentheses in readings (`expandReadingAlternatives`, `expandParens`), decided by **syllable count = headword length** (Hanji, □, Bopomofo count; punctuation and ASCII do not):

- ` / ` with spaces (HakSpring's MOE display column) → whole-reading alternatives.
- Tight `a/b`: if the tokens already match the headword length, each token's `/` is that syllable's alternatives (開始 `kói-cìi/sìi` → `kói-cìi`, `kói-sìi`; product over syllables); else if every `/` piece matches, whole-reading alternatives (溜走 `tiu-zèu/liu-zèu`).
- Twice the headword length → two whole readings joined by `-`, then the syllable `/` expansion (佢等 `ǐ-dén-ǐ-nén`; 哪仔 `nai-è-lai/nai-é` → `nai-è`, `lai-é`, `nai-é`).
- Neither → printed at the end of the build, not guessed (`/` rows are dropped).
- Headword with a parenthesised synonym and a reading without parentheses (usually a 【】 variant): the reading goes to whichever side its syllable count fits (調羹仔（湯匙）【tiǎu/těu gáng è】 → 調羹仔 only; 帖（帖仔）【tiàb è】 → 帖仔 only). A side with no countable characters (digits) keeps the old behaviour: both.
- Headword without parentheses, reading with them: a parenthesis glued to a syllable is that syllable's alternative (`ngǎi dén(nén)`); a spaced one is a whole-reading alternative when its syllable count equals the headword length (`ngǎi dén (ngǎi nén)`), else a one-syllable alternative of the syllable before it (`ngǎi dén (nén) tai gá`).

- `、` lists (`splitListedPair`): headword items and reading items pair by position (今下、現下 `gím/gín-ha、hien-ha`); unequal counts are printed, not guessed. A headword with `，` is one saying: not split, punctuation dropped from the reading.
- Reading-only 【】 variants (`assignVariantReadings`; `variant-parser` gives `word: ''`): a variant is the same word in another sub-lect, so each variant reading (after splitting its parentheses, then its `、`) goes to the main headword with the same syllable count whose main reading is closest (toneless edit distance). 盡尾（第一尾、最尾）【zùi mí】 → 最尾; 本來（原本、原來）【ngiǎn bùn、ngiǎn lǒi】 → 原本, 原來; 打單（買票）【mái peu】 → 買票. A tie goes to all tied headwords only when their main readings are identical (屋下（屋家）`vùg ká`【lùg ká】), else it is printed. Variants with their own headword (美濃 【皺波螺風】) are processed as their own line.

Readings that differ only in tone still collapse into one row (except MOE 特, above).

Build HakSpring data from its `main` (`HAKSPRING_ROOT` unset). Still open (Obs `IME` C8): `/` and `、` in readings and headwords, `…` / `－－` / digits in headwords, stray `ˊˇˋ` in certification readings.

## Sub-lect readings (planned)

Derived south Sixian / Meinong / Zhuolan / Taoyuan readings are produced by HakSpring, not by Kipan: an attachment table `data/sub-dialects-fill/<lect>.json` keyed by certification file + number (MOE: lect + headword). Kipan reads it, marks the variant bits and discounts the frequency of derived readings; it does not derive readings itself. Rows HakSpring could not derive are listed separately and must not be read.

## Verification done for P0

- Rebuilt dictionary vs. the previous one: same 23,124 records in the same order with identical headwords, readings, frequency and walker cost; `dictionary.fst` and `syllables.fst` byte-identical. Only `bitmask` (certification-only 4,516 rows → `2`, both 4,931 → `3`) and the subtag lect bits differ.
- Default-settings candidates for 24,166 inputs (every toneless key, every abbreviation, 800 random 2–3 word strings) are identical before and after, through `taigi-desktop-core`'s engine.
- `engine_roundtrip.rs` `hakfa_cert_is_its_own_source`: 護理師 (`fulisii`, certification only) survives MOE off and disappears with bit 1 off at the engine.
