# Candidate list: where Kipan differs from upstream

The continuous candidate list is: slot 0 = the walker's whole-buffer best path, then the left-anchored spans (`(0, end)`) of the lattice, each with its words. Which spans are listed is decided in `engine/composing/src/shadow.rs` `left_anchored_keys_and_restrictions`.

## Straddled shorter syllable (§18 guard (e)) — on for TL

Upstream drops a single syllable that is a strict prefix of a longer single syllable (`ta` beside `tai`), unless (c) a phrase also ends there, or (e) one syllable leaving its end runs past the longer syllable's end. Upstream applies (e) to TPS only; TL and POJ keep dropping.

Kipan types Hakfa in TL mode, and Hakfa entering-tone codas `b d g` are also onsets. Typed with one `d`, `hedui` parses both `hed|ui` and `he|dui`. Without (e) only `hed` (核 歇 黑) is listed after slot 0, and `he` (係) cannot be picked at all (maintainer report 2026-10-19, `iaheduioigefibo`). Kipan turns (e) on for TL; POJ stays upstream.

Typed with both `d` (`heddui`), nothing straddles and `he` stays dropped.

Effect on the Sixian dictionary (all toneless keys in `hakspring.csv` plus 2,000 random 2–3-word concatenations, 19,969 inputs, default settings):

- 2,249 lists change; slot 0 never changes; no candidate is lost; the old candidates keep their relative order.
- The change only adds the shorter span's words, which interleave with the longer span's by rank (e.g. `aba`: 掗 阿 now sit among 押 鴨 壓).

Code: one line in `shadow.rs` (`InputMode::Tps | InputMode::Tl`); upstream test `straddled_prefix_guard_stays_tps_only` renamed `straddled_prefix_guard_tps_and_tl_not_poj` and flipped for TL; Kipan test `hakfa_shorter_syllable_listed_when_coda_is_next_onset` in `desktop/crates/taigi-desktop-core/tests/engine_roundtrip.rs`.

Merge note: both touched spots are upstream code; a conflict there means upstream changed §18 — re-read the rule before resolving.
