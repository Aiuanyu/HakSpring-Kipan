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

Lect codes: `1` Sixian (四, `xiien`), `2` Hailu (海陸, `hoiliug`), `6` South Sixian (南, `namxiian`: its own lect, built from the MOE 南 volume, the 南 supplement and the certification list's South readings (below); the settings list it second, never as a sub-choice of Sixian). Reserved: `3` Dapu, `4` Raoping, `5` Zhao'an. `0` means "no lect" and fails the build.

The engine reader keeps bits 13–15 (`KAUTIAN_SUBTAG_LECT_MASK`) in `DictionaryRecord.kautian_subtag`. They sit outside `KAUTIAN_SUBTAG_USED_MASK` (bits 0–12), so they never reach the subcollection AND. `DictionaryReader::passes_filter` has a lect gate: when the filter's `lect` is non-zero, a record whose lect is non-zero and different is dropped. A record with lect 0 (non-Hakfa data) and a filter with lect 0 (or the `u32::MAX` sentinel) match everything.

The wanted lect rides **wire bits 27–29** of `enabled_sources_bitmask` (`WIRE_LECT_SHIFT`, `WIRE_LECT_MASK`; `0` = all lects, the behaviour before lects). Bits 30–31 are still free. If upstream starts using 27–29, move the lect to its own settings field; the change is confined to `Filter::from_enabled_bitmask` and the platform encoders.

## One merged dictionary

`node dictionary/sources/official/hakspring/extract_and_build.js` (no argument, same as `--lect all`) runs the extraction once per lect (child process `--dump-records`), concatenates, sorts by frequency (ties: Sixian first) and encodes one `dictionary.bin`. That is the shipped dictionary: it overwrites `assets/dictionaries`, `output/hakfa`, `dictionary/output/hakfa` and writes `dictionary/output/hakfa-all`. `--lect 四` and `--lect 海` build one lect alone for debugging and write `dictionary/output/hakfa-<id>/` and `data/hakspring-<id>.csv` only. Lect ids are the Hakka names: `xiien`, `hoiliug`.

Build from the HakSpring (客源翠) `main` data: set `HAKSPRING_ROOT` to a checkout of `main` when the usual checkout sits on a feature branch (its `data/supp` can carry rows `main` does not have yet).

Test: `engine/lexicon/tests/hakfa_lect.rs` opens the merged dictionary (skips if not built) and checks that lect 1 + lect 2 = every row, lect 0 = every row, lect 3 = none.

## The lect setting

- desktop-core: `Lect` (`settings/choices.rs`, stored `xiien` / `hoiliug`), key `lect` (`settings/keys.rs`), default Sixian. It is a key-path setting, so macOS carries it in the settings snapshot; `dictionary_toggles` sends `DictionarySourceToggles.lect = Lect::code()`. The General pane's reset does not touch it.
- macOS: `Lect` and `SettingsStore.lect` (Swift twins), shortcut `cycleLect` (**⌃⌘L**, first row of the Shortcuts pane and the input-method menu), the mode flash names the lect (`四縣 xiien` / `海陸 hoiliug`). An open candidate bar is fetched again.
- Which lects can be typed in: keys `lectEnabled.<id>` (`keys::lect_enabled`), one per lect, switched in the Dictionary Sources pane under the MOE dictionary (macOS `DictionaryTogglesView`). `SettingsDocument::available_lects` / `effective_lect` resolve the lect the engine gets: the stored one, or the next available one when its row was switched off (the stored choice is not rewritten). The cycle shortcut steps through the available lects only. The last lect on cannot be switched off. The certification rows under it are display-only and follow the MOE rows (certification Sixian is on if MOE Sixian or South Sixian is). No engine change: the engine still gets one lect.
- Android: sends lect 1 (`LexiconBridge.kt`) until it has a setting; lect 0 would mix every lect.
- Windows / Linux: the lect setting exists in desktop-core (default Sixian) but they have no shortcut or picker for it yet.

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

## Learning data across lects (各腔互學)

Learned words and phrases live per lect (`user_frequency.<id>.db` and so on; 四縣 keeps the original names). `UserDataRequest.sync_lects` offers what each lect learned to the others (`engine/dispatch/src/user_data/lect_sync.rs`): a word is copied to a target lect only when the target's dictionary has the same Hanji with exactly one reading. A Hanji the target does not have, or reads several ways, is skipped, never guessed. A copy counts as one use and never raises what the target learned itself. Everything is read before anything is written, so a copy never echoes back. `dry_run` counts only (the preview). Answered in `dispatch`, not `userdata`, because it needs the lexicon. macOS: a row on the Learning Records page asks for the preview, then confirms. Not copied yet: next-word associations.

## South Sixian and the Sixian certification list

The certification list has no South Sixian file; its Sixian lines mark South readings with `【】` (Meinong readings with `【（）】`). The build splits them (`linePairsSplit`):

- **四縣 (1)**: only the main (North) reading of each line. The `【】` readings are South Sixian, so they are no longer Sixian candidates (`就 lu` used to show up under 四縣).
- **南四縣 (6)**: for each Sixian line, in order: the `【南】` reading(s) when the line has them (the North reading stays at 0.98 of the frequency); otherwise the derived reading from HakSpring's `sub-dialects-fill/南四縣.json` (`來源檔|編號`; layers other than the MOE South volume get 0.8 of the frequency; the `待確認` single-character rows are left out); otherwise, if the line is in `待審-南四縣.json` (a syllable with no South evidence), the North reading stands in at half frequency (`SOUTH_INCLUDE_PENDING`), so a word every Sixian user has (繼續) is not missing from South Sixian; a real South reading, once reviewed and moved into the table, outranks it.
- **Meinong readings** (`【（）】`, 35 records) are South Sixian rows flagged with subtag bit 3 (`KAUTIAN_SUBTAG_MEINONG_BIT`, upstream's accent slot 3, unused by Hakfa records). They pass only while the Meinong switch is on (`Filter.meinong`, wire bit 30, `DictionarySourceToggles.meinong`, setting `meinongEnabled`, **off by default**). Meinong is a switch under 南四縣 on the Dictionary Sources pane, not a lect of its own (maintainer, 20261025). A record with no lect is never hidden by it. Not done yet: the derived Meinong table (`sub-dialects-fill/美濃.json`, tone 24→33) as an output option.

A golden-corpus sentence can name its lect (`lect` in `golden_corpus.json`; default `xiien`); the two South Sixian sentences (街仔 `giai`, 在尊嚴…) do.

## South Sixian gaps filled from North Sixian

The maintainer's principle (20261025): what South Sixian lacks is filled from North Sixian when the North data can be taken and does not conflict with South data. In the merged build (`--lect all`), a North Sixian word that South Sixian has **no reading of at all** (810 words) is copied into South Sixian at half frequency (625 words, 635 rows from the MOE volume). **Supplement (補字典) rows are not copied**: the single-character supplement and its sub-dialect derivation have their own handling and principles in HakSpring (`dict-augmentation`), and a character missing from the South supplement is that pipeline's to decide, not this fill's. A word South Sixian has in any reading is never touched: a different North reading of it (`kiěn` vs `kiǎn`, or only a different tone) is a North/South difference, not a gap, and copying it would contradict South data. The copied words are derived toward South Sixian: the regular `-ien/-ied → -ian/-iad` after velar (g, k, ng), glottal (h) and zero initials (tone mark moves to the `a`), and a per-character table learned from the words both lects have (a character's North syllable → South syllable, used when seen at least twice and at least 80% of the time); a syllable neither covers keeps the North reading. Checked leave-one-out on the words both lects have: whole word right 95.5% (copying the North reading as is: 91.0%), syllable 97.8%. 52 of the 635 copied rows changed.

## The 「美」 mark

A candidate whose record is a Meinong reading carries `variant = 1` (`RawCandidate.variant` → `CandidateMessage.variant` → desktop-core `ContinuousCandidate.variant`); the candidate window writes 「美」 after its romanization (`roman_for_display`; display only, the commit is the engine's `display_text`). With the Meinong switch on, a Meinong reading is ranked just above the ordinary South reading of the same word (frequency ×1.02) and both stay (the maintainer's choice: coexist, mark 美). A Meinong reading identical to the ordinary one is merged away at build time. The Windows, Linux and Android candidate windows do not draw the mark yet.

## Meinong tones (24 → 33)

Writing tone 24 as 33 is its own option, `tone33Enabled` (off by default; maintainer's ruling R2-16 and USER 2026-10-25: a function Meinong speakers choose on their own, apart from the Meinong readings switch `meinongEnabled`). Meinong speakers read the 24 syllables of Sixian and South Sixian as 33, so the romanization is written that way. The Dictionary Sources pane has it in its own 「輸出選項」 section. `AppConfig.meinong_tones` (proto tag 16, set from `tone33Enabled`) makes `composing/requests.rs` rewrite the display romanization after the candidates are assembled: acute `á é í ó ú ḿ ń` become macron `ā ē ī ō ū m̄ n̄` (`phonetics::api::tone24_to_33`), for lect 0, 1 and 6 only (`AppConfig::renders_meinong_tones`). Only `roman` changes: `canonical_tl` (the identity the frequency, learning and associations key on), `hanji` and the dictionary stay as they are, and nothing is sandhi-rewritten, since Meinong does not apply the 24-triggered sandhi. The committed romanization is written the same way, because a commit sends the candidate's own `roman` (the 「美」 mark is only added to the cell). The derived Meinong table (`sub-dialects-fill/美濃.json`) is not read: 98.9% of its rows are exactly this tone rewrite, the rest are the lexical differences, which the Meinong-flagged records carry. Hailu and the other lects are untouched.
