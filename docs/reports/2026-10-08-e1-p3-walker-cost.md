# E1 P3 — the walker prices edges on `walker_cost`

> **Date**: 2026-10-08 · **Plan**: [`architecture/unified-word-frequency-roadmap.md`](../architecture/unified-word-frequency-roadmap.md) P3 · **Baseline**: [P1 report](2026-10-08-e1-walker-baseline.md), [corrections](2026-10-08-e1-walker-baseline-corrections.md)

## Summary

- The walker prices each lattice edge on the lowest `dictionary.bin` v4 `walker_cost` among the key's visible homophones (D2 + D3 (c)). Which word fills the edge is unchanged.
- `kau3siu7` → 教授 at slot 0 (was 到受). Engine, TL full tone: `dev` exact 81.4 → 86.4 %, segmented 92.4 → 97.0 %; `calib` exact 80.9 → 85.3 %, segmented 89.7 → 95.6 %. Every input variant improves; no category drops.
- Custom entries, learned phrases and selected words share one cost cap. `CUSTOM_EDGE_COST` = 7,878 milli-nats: the model's price of a word seen 2,000 times (the old effective frequency).
- `CORPUS_TOTAL_FREQ`, its guard test, `corpus_total_freq.txt` and the two test skips are retired.

## 1. Engine metrics (`walker_gold.rs`, default sources)

Exact % / segmented % / mean rank. Before = P1 baseline (`dev` TL full from the corrections report).

| Split | Variant | n | Before | P3 |
|---|---|---|---|---|
| dev | tl_full | 264 | 81.4 / 92.4 / 0.55 | 86.4 / 97.0 / 0.46 |
| dev | tl_partial | 264 | 78.1 / 88.3 / 0.64 | 85.6 / 96.2 / 0.47 |
| dev | tl_toneless | 264 | 60.4 / 63.8 / 1.37 | 70.5 / 76.1 / 1.08 |
| dev | poj_toneless | 264 | 60.8 / 64.5 / 1.38 | 69.7 / 75.4 / 1.11 |
| dev | tps_full | 264 | 74.0 / 81.1 / 0.85 | 82.2 / 90.5 / 0.66 |
| dev | tps_toneless | 264 | 61.9 / 65.7 / 1.30 | 71.6 / 77.3 / 1.01 |
| calib | tl_full | 68 | 80.9 / 89.7 / 0.90 | 85.3 / 95.6 / 0.74 |
| calib | tl_toneless | 68 | 57.4 / 58.8 / 2.07 | 61.8 / 64.7 / 1.85 |
| calib | tps_full | 68 | 73.5 / 77.9 / 1.26 | 77.9 / 85.3 / 1.10 |

`dev` n was 265 in P1; one item became unsupported (corrections report §1). TL full tone by category: `common`, `rare`, `unseen`, `neutral` 100 % exact; `phrase` 81.7 % (`calib` 82.8 %); `variant` 60.0 % exact, 100 % segmented — its misses are the edge pick (P5), not segmentation. The toneless strata improve too: D3 (c) lets a toneless key borrow its cheapest tone, which the P1 caveat flagged; the gold set shows no loss from it.

Simulator (`tools.walker_gold simulate`, A2 at α 10) vs the engine on TL full tone: 320 / 332 agree on hanji and displayed word lengths (pre-P3 model before P3: 319 / 332). The disagreements are displayed word lengths only (`哪會` shown as two words by the engine).

## 2. Custom / learned / selected cap

In nats, before the khiin length terms: `base = min(walker_cost / 1000, cap(s))`, `cap(s) = CUSTOM_EDGE_COST / 1000 + ln((1 + 2000) / (1 + 2000·s))`, `s = δ / BOOST_ALPHA` clamped to 0..1 (exact translation of the old frequency floor). A custom edge is priced at the cap with `s = 1`; an edge with a learned phrase is capped at it.

Anchor rule (unchanged from the 2,000 provenance): a two-syllable custom entry beats the split into the two cheapest single characters (7.878 < 的 4.173 + 伊 4.400 at the shortest spelling), and a one-syllable custom entry never undercuts the cheapest single character (7.878 / len^0.2 > 4.173 for len ≤ 23). Rejected anchors: the same rank among single characters as the old 2,000 (10.05 nats — loses to 的 + 伊 for a two-letter reading) and just under the cheapest multi-syllable word (≈ 5.9 nats — close to the top single characters). At `s = 0` the cap is 15.48 nats, above the unseen cost 13.18: no lift.

## 3. Behaviour changes in the tests

- **S0 golden** (hermetic, `golden_fetch_at_pos.rs`): fixtures keep their pre-E1 cost (`test_support::walker_cost_from_fixture_frequency`, `round(ln(13,056,588 / (1 + freq)) × 1000)`), so dictionary edges move only by the milli-nat quantisation (≤ 0.0005 per edge in the score column). Two custom-edge rows move with the new anchor (`taigi` 台語 −7.3126 → −6.5589; `taigikhipoann` −6.9388 → −6.2236), hanji unchanged. `taigilangkangtanlai` shows `tâi-gí langkangtanlai` (was `tâi-gí lang kangtanlai`): every all-OOV split of the tail costs the same 14 × 10¹⁰, and the old result came from floating-point rounding of a different dictionary edge cost; the walker now keeps the first-reached tail, as its tie rule states.
- **TPS conversion** (`tps_hanji_conversion_prod.rs`): `ㄍㄧㄣ ㄚˋ` converts to the word 巾仔 (cost 12,348 → 10.75 with the length terms) instead of 今 + 仔 (10.402 / 3^0.2 + 7.643 = 16.0; the `kin` edge borrows 根's 10,402 under D3, 今's own is 10,632); closing ㆢㄧㆵ˙ still gives 今仔日.
- **Fixed regression inputs** (`walker_fixed_inputs_prod.rs`, default sources): `kau3siu7` 教授, `tai5gi2` 台語, `hoogua` 予我 `hōo--guá`, `taiuan` 台灣, `ginalangtsiahpngbesai` 囡仔人食飯袂使 (食飯 is outside the default sources, so 食 and 飯 show as two words).

## 4. Seen while measuring (not changed here)

- With every source on, `taiuan` shows 台員 at slot 0 and 台灣 second, on `main` too: the khiin-only row 台員 shares 台灣's frequency 1,379, and the walker's edge pick prefers it while the list ranks 台灣 first. The default sources hide 台員.
