# TaigiKeyboard — Roadmap

> **Type**: Planning (forward-looking)
> **Keywords**: `roadmap`, `planning`, `open candidates`
> **Status**: Active

---

## Summary

- **Forward-looking work only.** Shipped versions: [`CHANGELOG.md`](../CHANGELOG.md) (release notes per train); the v3.5.8 plan archive is [`releases/v3.5.8/plan.md`](releases/v3.5.8/plan.md). Shipped designs live in the `architecture/*-roadmap.md` docs and in `architecture/behavioral-invariants.md`.
- Pending device dogfood: [`architecture/dogfood-checklist.md`](architecture/dogfood-checklist.md).
- Release scope, timing and tags are the maintainer's call; nothing below is assigned to a release.

---

## Active / In-flight items

- **Maintainability audit follow-up (2026-10)** — [`architecture/maintainability-roadmap-2026-10.md`](architecture/maintainability-roadmap-2026-10.md), rounds H, A–G over [`reports/2026-10-09-audit-all.md`](reports/2026-10-09-audit-all.md). In progress: H, A1a (#473), A1b (#475), A2 (#490) merged; next A3.

---

## Open candidates (unscheduled)

One line each; the linked section holds the design, the measurements and the open questions.

**Project and legal (maintainer)**

| Item | Status | Where |
|---|---|---|
| SignPath Foundation code-signing application | rejected 2026-10-02; to be reapplied later — keep `CODE_SIGNING_POLICY.md` + the `windows-build.yml` provenance build | [`CODE_SIGNING_POLICY.md`](CODE_SIGNING_POLICY.md) |

**Engine and dictionary**

| Item | Status | Where |
|---|---|---|
| Retire the old dictionary `frequency` after E1 | Recorded, not started. Still read: the equal-cost tie-break (`CandidateSortKey` −score / −freq; ~69 % of rows are corpus-unseen and share one cost), the search-page tie-break, `association.bin` dictionary-derived counts (`associations.py`), CSV / FST row order + duplicate max-aggregation (`merge_csv.py`), the `dict.bin` u32 field, `CandidateMessage.score`. Steps: pick a replacement tie-break for unseen words, a count source for the association rows, then the format bump. A custom twin of a dictionary word carries no corpus cost (sorts below its priced homophone in the list) | [`reports/2026-10-08-e1-p5b-list-sort.md`](reports/2026-10-08-e1-p5b-list-sort.md) §5 |

**Mobile parity (inventory 2026-10-12)** — pending code; directions decided unless marked.

| Item | Direction | Where |
|---|---|---|
| iOS settings reset drifts from declared defaults | reset clears every owned key except user themes (today misses `isCustomDictEnabled`, `inputModeBeforeTps`, `layoutBeforeTps`) | `Settings/SharedSettings.swift` `resetToDefaults` |
| Android setup guide shows iOS steps (Add keyboard, Allow Full Access, globe) | Android-scoped i18n keys for steps / closing / info note | `i18n/home.json`, `ui/tabs/home/SetupGuideScreen.kt` |
| Android copyright list lacks the STTI (學科術語) OGDL credit | add entry | `content/CopyrightData.kt` |
| Enter while composing in an action field (Send / Search) | Android matches iOS: first Enter confirms only, second runs the action; no auto-space in action fields (both) | `TextInputKeyHandler.kt`, `ActionHandler+KeyActions.swift` |
| Double-space period | both, Taigi and English, when not composing; full-width inserts `。` without a space; iOS gated by KeyboardKit `isPeriodShortcutEnabled` | same |
| English suggestion pick | both obey Auto-Space (`shouldAppendAutoSpace`, §23 swap arm) | `ActionHandler.swift`, `CandidateClickHandler.kt` |
| Single-shot Shift | iOS matches Android: only cased letters consume it | `ActionHandler+KeyActions.swift` |
| Android spacebar swipe | add swipe-to-move-cursor, keep long-press picker | `KeyTouchCoordinator.kt` |
| Layout | Android: English mode loads TL callouts, globe left of space; iOS: `…` for ellipsis, `@` / `/` by `keyboardType`, return key by `returnKeyType` | layouts / callouts |
| Small | iOS: Learning Records Count / Last Used labels, drop strip cap 200, localize backup / CSV import errors; Android: named long-press a11y action, keycap-hint invariant test, no "No words yet" flash, no duplicate setup guide | per item |
| Needs device repro first (UNCONFIRMED) | Android symbols-page `、` / numeric pad wipes the preedit; Android next-word predictions survive a cursor move | `+Cleanup.swift`, `TextInputManager.kt`, `TextInputKeyHandler.kt` |

## Not to re-propose

Closed by the maintainer; one line each.

- `zh-TW` README; xcconfig signing; Android Gradle proto plugin; dictionary-source licensing follow-up.
- Romanization spelling correction (fuzzy / edit-distance tolerance in the engine) — dropped when the Candidate Display = Romanization mode shipped without it.
- Corpus expansion P1b, including the TAT application.
- A full TL / POJ / en / ja proofreading pass of the app UI — reported typos are fixed per incident.
- `$` sentence-start opener (its `association.bin` data is removed in its own PR).
- Smart-suggestion techniques outside the bigram model: 3 zero-query continuation from learned history, 5 negative feedback on rarely picked entries, 6 zero-query rule table (`「`→`」`, number → measure word), 9 word → emoji, 10 offensive-word filter, 11 privacy gates (incl. the Android `IME_FLAG_NO_PERSONALIZED_LEARNING` gap). Names of all eleven: [`architecture/bigram-lm-roadmap.md`](architecture/bigram-lm-roadmap.md).
- Windows candidate-window paint latency (frame drawn before its candidates on a busy host UI thread) and UIA exposure.
- Converted-romanization commit on Enter (`suann2ting3` → `suán-tìng`).
- TL mode (臺羅模式) tone key commits the syllable without a candidate window (MOE Mac IME parity).
- Mobile predictions after Space.
- Bigram P6 walker term, P7 hanji-only sources, D7 two-word context.
- e2e drivers for macOS, Windows, Android, iOS.
- Invariant-label PR2; naming batch C (persisted names stay frozen).
- Shared Swift package for macOS + iOS — see [`architecture/maintainability-roadmap-2026-10.md`](architecture/maintainability-roadmap-2026-10.md) § Best practices alignment (deliberately not adopted).
