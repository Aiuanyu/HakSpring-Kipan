# E1 P2b — Corpus Subsegment Tie-break

> **Type**: Report (dated snapshot — frozen)
> **Keywords**: `walker`, `word_unigrams`, `word_bigrams`, `subsegment`, `association.bin`, `exposure`
> **Date**: 2026-10-08 · branch `feat/e1-p2b-subsegment-tiebreak` (E1 P2b) off `main` `9bf3de69`
> **Plan**: [`architecture/unified-word-frequency-roadmap.md`](../architecture/unified-word-frequency-roadmap.md) §6 P2b · **Corrects**: [`2026-10-08-e1-walker-baseline.md`](2026-10-08-e1-walker-baseline.md) (left as published)

`dictionary/build/corpus_bigrams.py` splits an out-of-vocabulary corpus word into dictionary words: fewest pieces, then a tie-break. The tie-break was the highest summed `DictionaryRecord.frequency`, the mixed-scale value E1 replaces. P2b counts the corpus twice. Pass 1 counts only the words found as written and leaves out-of-vocabulary words unsplit. Pass 2 breaks fewest-piece ties by the lowest sum of the pieces' quantised milli-nat costs under a model frozen from pass 1. The engine is unchanged; mobile next-word (`association.bin`) changes.

## Summary

- **The committed counts were not stale.** Regenerating with the code on `main` reproduces `word_unigrams.tsv` and `word_bigrams.tsv` byte for byte (43 s). The P1 report took the log's `taigi_bible_nt` `tok:in-vocab` 334,126 as the column total. That stat leaves out the 73,549 `tok:romanized-mapped` tokens: 334,126 + 73,549 = 407,675, the TSV column. The "refresh" half of P2b was a no-op; the maintainer approved the slimmed phase on 2026-10-08.
- **3,211 of 224,046 split tokens (1.4 %) change their split.** Every per-source token total, `tok:subsegmented` and `tok:in-vocab` count is identical to the old run, because fewest pieces stays the primary key. Only which words receive the counts changes.
- **`association.bin`: 163 of 21,562 word keys change their first next word**; 148 keys disappear, 33 appear. The largest losses are split artifacts (教 → 會報, 中 → 國字, 五 → 十箍), with some real words among the removed keys (水雞, 查某嬰).
- **Walker model**: 2,360 of 167,907 `walker_cost` values change. Simulated A2 on `dev` / `calib` is unchanged at every α, and the exposure screen moves by at most 8 keys.

## 1. Inputs

| Item | Value |
|---|---|
| Repository | `main` `9bf3de69` |
| `corpus/taigi-corpus` / `corpus/taigi-typing` | `5ba95091` / `3bd508ee` |
| `output/dictionary.csv` sha256 | `10c8ee1ebac70a4a…` (unchanged by this phase) |
| `word_unigrams.tsv` sha256 old → new | `bdfe328a01744f0f…` → `c0d3051d745bccb2…` |
| `word_bigrams.tsv` sha256 old → new | `240e5ecba020aec1…` → `c27e4f1b73ff94e2…` |
| Sources | all 9 (the default) |
| Tie-break model | pass-1 counts without `taigi_typing`; N = 3,149,114, V = 167,445 (the lexicon's model words), `SUBSEGMENT_ALPHA` = 10, fixed apart from the walker's α so tuning the walker in P4 does not regenerate the corpus |
| Runtime | 71 s (two passes) |

## 2. Counts

| | Old | New |
|---|---|---|
| Split OOV tokens (Σ `tok:subsegmented`) | 224,046 | 224,046 |
| of which split differently | – | 3,211 |
| Unigram rows | 52,017 | 51,844 |
| Bigram rows (count ≥ 2) | 283,275 | 283,262 |
| Unigram tokens N (all 9 sources) | 3,629,550 | 3,629,550 |
| Model words whose count changes | – | 2,370 (1,175 up, 1,195 down) |

Most frequent changed splits (old → new, tokens):

| Old | New | Tokens | Reading |
|---|---|---|---|
| 教 會報 | 教會 報 | 313 | better |
| 五 十箍 (also 二 / 三 / 四 / 六 十箍) | 五十 箍 | 40 (+75) | better |
| 公 報社 | 公報 社 | 35 | better |
| 兩百 萬 | 兩 百萬 | 34 | either |
| 中 國字 | 中國 字 | 34 | better |
| 臺 中縣 | 臺中 縣 | 31 | better |
| 臺南市 長 / 高雄市 長 | 臺南 市長 / 高雄 市長 | 36 | better |
| 不 如意 | 不如 意 | 24 | worse |
| 水雞 仔 | 水 雞仔 | 20 | worse |
| 查某嬰 仔 / 樹葉 仔 | 查某 嬰仔 / 樹 葉仔 | 28 | worse |

A unigram tie-break prefers a very frequent single character next to a rarer word (水 雞仔). The Codex pre-impl review kept it anyway: a piece-length or single-character rule would be a new heuristic with no evidence behind it, and it would weaken the probability comparison.

## 3. `association.bin`

Compared semantically from `associations.compute_word_associations` over the old and new `word_bigrams.tsv` (the `build_ts` header changes with the counts and is not a content change).

| Change | Word keys |
|---|---|
| Keys before → after | 21,677 → 21,562 |
| Keys removed / added | 148 / 33 |
| Next-word set changed | 711 |
| Same set, order changed | 156 |
| Same order, counts only | 133 |
| First next word changed | 163 |

Removed keys, largest first: 會報, 國字, 水雞, 查某嬰, 宜蘭縣, 公議, 三十分, 中縣, 報社, 跳出. Added: 親生, 電纜, 雞仔, 梅仔, 年仔.

Common keys whose first five changed:

| Key | Old first five | New first five |
|---|---|---|
| 教 *kàu* | 會報 的 界 會 勢 | 的 界 會 勢 聯 |
| 中 *tiong* | 的 有 來 所 國字 | 的 有 來 所 咧 |
| 五 *gōo* | 年 个 六 日 十箍 | 年 个 六 日 歲 |
| 教會 *kàu-huē* | 的 公報 有 內 是 | 的 報 公報 有 內 |
| 台灣 *tâi-uân* | 教會 的 人 教 文學 | 教會 的 人 文學 是 |
| 不 *put* | 成 知 如意 見 可能 | 成 知 見 可能 容易 |
| 查某 *tsa-bóo* | 囡仔 囝 人 無 毋 | 囡仔 囝 人 無 嬰仔 |
| 个 *ê* | 人 是 囡仔 真 所在 | 人 是 囡仔 真 查某 |

`association.bin` also feeds composing-candidate re-ranking by the previous word (`engine/dispatch/src/context.rs:57`), so set changes matter there as well as in the next-word strip. Dogfood: `dogfood-checklist.md` S116.

## 4. Walker model and measurements

`output/walker_lm_stats.txt`: `records_with_count` 52,298 → 52,126, `cost_min` 4,172 → 4,173; N, V, α, the unseen cost and the percentiles are unchanged. The walker still prices on `frequency` until P3, so the engine baseline (`walker_gold.rs`) is unchanged (`dev` TL full 81.4 % / 92.4 %, `calib` 80.9 % / 89.7 %).

Simulated A2 (`walker_gold simulate`, `dev` + `calib` only): every stratum and every α matches the [corrections report](2026-10-08-e1-walker-baseline-corrections.md) §5 to the printed percent.

Runtime D3 screen: default 930 / 195 (was 930 / 193), all sources 3,993 / 755 (was 3,993 / 752).

Exposure (`walker_gold exposure`, 39,482 keys):

| Model | Word loses | with a corpus count | word → split | split → word |
|---|---|---|---|---|
| current | 1,485 | 1,323 | – | – |
| A2 α 0.5 | 1,369 (was 1,365) | 558 (554) | 1,067 (1,059) | 1,183 (1,179) |
| A2 α 2 | 705 (703) | 405 (404) | 442 (442) | 1,222 (1,224) |
| A2 α 10 | 222 (223) | 147 (148) | 68 (68) | 1,331 (1,330) |
| A2 α 50 | 24 (24) | 17 (17) | 1 (1) | 1,462 (1,462) |

## 5. Reproduce

```sh
cd dictionary
python3 -m build.corpus_bigrams          # both passes; log in logs/corpus_bigrams.log
cd .. && make dict && make build
cd dictionary
PYTHONPATH=. python3 -m tools.walker_gold resolve
cd ..
GOLD_OUT=/tmp/baseline.tsv GOLD_SLOT0_OUT=/tmp/slot0.tsv \
  cargo test --manifest-path engine/Cargo.toml -p composing --test prod walker_ -- --include-ignored --nocapture
cd dictionary
PYTHONPATH=. python3 -m tools.walker_gold d3
PYTHONPATH=. python3 -m tools.walker_gold simulate --engine-slot0 /tmp/slot0.tsv
PYTHONPATH=. python3 -m tools.walker_gold exposure --alpha 10   # and 0.5, 2, 50
```
