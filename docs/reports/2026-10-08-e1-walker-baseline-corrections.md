# E1 Walker Baseline — Corrections (P2)

> **Type**: Report (dated snapshot — frozen)
> **Keywords**: `walker`, `gold set`, `baseline`, `D3`, `exposure`, `correction`
> **Date**: 2026-10-08 · branch `feat/e1-p2-walker-cost` (E1 P2) · production artifacts rebuilt as `dictionary.bin` v4 (walker unchanged)
> **Corrects**: [`2026-10-08-e1-walker-baseline.md`](2026-10-08-e1-walker-baseline.md) (left as published) · **Plan**: [`architecture/unified-word-frequency-roadmap.md`](../architecture/unified-word-frequency-roadmap.md)

The P1 cloud review (landed after #460 merged) found three tooling defects in `dictionary/tools/walker_gold.py`. P2 fixes them and moves the simulator onto the P2 walker model (`dictionary/build/walker_lm.py`). This report re-runs every P1 measurement and lists what changed. The engine itself is unchanged, so every engine difference comes from the gold set losing one item.

## Summary

- **Gold set: 414 of 415 items** are measured. `dev-0388` 洗盪 is skipped and listed: its dictionary row has the non-ASCII `tl_num` `sere2tn̄g6` (one of 55 such rows), so the input variants could not be derived. The gold file and item ids are unchanged; `select` now excludes such rows.
- **Runtime D3 screen, default sources: 930 / 193, not 2,016 / 236.** P1 took the corpus winner from every row, including rows a default install cannot show. The conclusion holds: on 193 keys the engine's pick never occurs in the corpus while a visible homophone occurs ≥ 10 times, so D3 (c) stays.
- **Exposure: 1,059 / 68 dictionary words broken at α 0.5 / 10, not 1,077 / 69.** The A2 model now counts per `(hanzi, tl_num)` with V = distinct model words (167,446) and prices with the quantised `walker_cost`, as `dictionary.bin` v4 stores it.
- **`dev` numbers are in-sample for A2.** The `dev` sentences are 教典 example sentences, which the `moe_kautian` column counts. Only `calib` is held out from the A2 counts.
- **`final`**: P1 printed `final`'s aggregate simulator row once while the tool was being brought up (baseline §2). It has not been run through the engine. P4 reads it as planned and reports it as previously glimpsed.

## 1. What was fixed

| Finding | Fix |
|---|---|
| `resolve` exited 0 after skipping 330 / 415 items when `taigi-converter` was missing | An item that no longer resolves now exits non-zero and names the items and the submodules to check. Skips for a known reason (unsupported key) are listed with the reason. |
| `d3` default screen picked its winner among rows a default install cannot show | The winner is chosen among the rows visible under the screened sources. |
| Gold item `dev-0388` built from a row with a non-ASCII `tl_num` | `select` excludes rows whose `tl_num` is not `(?:[a-z]+[0-9]?)+`; `resolve` skips and lists such items. |
| Report §5 had no in-sample caveat for `dev` | See the summary above. |
| Roadmap P4 row (1,064 / 65) disagreed with the report (1,077 / 69); the plan said `GOLD_SPLIT` | Roadmap now quotes this report; the variable is `GOLD_SPLITS`. |

Not changed: the review asked for `unittest` in the test docstrings because pytest was missing in its VM. pytest is this repository's runner (`.github/workflows/python.yml`), so the docstrings stay.

## 2. Reproduce

```sh
git submodule update --init corpus/taigi-typing taigi-converter
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

## 3. Engine baseline (TL full tone)

| Split | n | exact % | segmented % | boundary F1 | top5 % | mean rank |
|---|---|---|---|---|---|---|
| dev (was 265) | 264 | 81.4 (81.1) | 92.4 (92.1) | 0.952 (0.949) | 90.9 (90.6) | 0.55 (0.57) |
| calib | 68 | 80.9 | 89.7 | 0.962 | 82.4 | 0.90 |

`calib` is unchanged on every variant. Other `dev` variants moved by at most 0.4 points; `dev` / `unseen` has 39 items.

## 4. Runtime D3 screen

| Sources | Keys | Pick ≠ corpus winner | Pick unseen, winner ≥ 10 | Top examples (pick (count) → winner (count)) |
|---|---|---|---|---|
| default | 14,849 | 930 (was 2,016) | 193 (was 236) | 傳導 (0) → 傳道 (1,611) · 二億 (0) → 利益 (1,135) · 毋閣 (0) → 毋過 (898) · tsiânn做 (0) → 成做 (658) |
| all | 14,849 | 3,993 (unchanged) | 752 (unchanged) | 今 (0) → 這 (20,489) · 因爲 (0) → 因為 (11,646) · 得着 (0) → 得著 (3,734) |

The `all` row screens every row, so the visibility fix leaves it unchanged. `hanji_counts` adds each model word once: two spellings that share a count (猴山仔 `kâu-san-á` / `kâu-san--á`) are not counted twice.

## 5. Simulated A2

Model: `build.walker_lm` — counts per `(hanzi, tl_num)` without `taigi_typing`, N = 3,627,713, V = 167,446, priced with the quantised milli-nat cost; min over the default-visible homophones; the engine's own pick.

| Stratum | n | current exact / seg | α 0.5 | α 2 | α 10 | α 50 |
|---|---|---|---|---|---|---|
| dev / all | 264 | 81 / 88 | 86 / 92 | 86 / 92 | 87 / 93 | 87 / 94 |
| dev / unseen | 39 | 100 / 100 | 97 / 95 | 97 / 97 | 100 / 100 | 100 / 100 |
| calib / all | 68 | 81 / 88 | 85 / 96 | 85 / 96 | 85 / 96 | 85 / 97 |
| calib / phrase | 58 | 79 / 90 | 83 / 95 | 83 / 95 | 83 / 95 | 83 / 97 |

The other strata match the P1 table. Simulator vs engine on the current model: 319 / 332 agree on hanji and displayed word lengths (P1: 320 / 333; the dropped item agreed). At α 0.5, 38 items change slot 0, as in P1.

## 6. Exposure

| Model | Keys where the word loses | of which the word has a corpus count | word → split | split → word |
|---|---|---|---|---|
| current | 1,485 | 1,323 (was 1,300) | – | – |
| A2 α 0.5 | 1,365 (was 1,395) | 554 | 1,059 (was 1,077) | 1,179 |
| A2 α 2 | 703 (was 723) | 404 | 442 (was 451) | 1,224 |
| A2 α 10 | 223 (was 234) | 148 | 68 (was 69) | 1,330 |
| A2 α 50 | 24 (was 25) | 17 | 1 | 1,462 |

The "with a corpus count" column rises because neutral-tone spellings now share their word's count. P2 builds `dictionary.bin` with the provisional α = 10; P4 still chooses α on `calib` phrases.
