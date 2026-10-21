# AGENTS.md

Working instructions for coding agents — and for humans — in this repository. Tool-specific additions live in each tool's own file (`CLAUDE.md` for Claude Code). Licensing, secrets and commit conventions: [`CONTRIBUTING.md`](CONTRIBUTING.md).

**HakSpring Kipan（客源翠企盤）** — a Taiwanese Hakfa input method: a soft fork of [TaigiKeyboard](https://github.com/taigikeyboard/taigikeyboard), the cross-platform Taigi IME (iOS / Android / macOS / Windows / Linux shells over a shared Rust engine). Kipan types Hakfa in HagPinPlus (HPP) — toneless continuous input, abbreviations, Hakfa Hanji — with dictionaries from HakSpring（客源翠）: the MOE Hakfa dictionary（教典）and the Hakfa proficiency-test vocabulary（認證詞彙）.

**Status**: macOS and Android are proofs of concept on Sixian（四縣）; iOS, Windows and Linux still ship upstream behaviour. Multi-dialect support is the next milestone. Most of the code and docs below still say Taigi / TL / POJ / TPS — that is upstream naming kept on purpose (§ Soft fork). In Kipan the `tl` fields carry the HPP reading.

## HakSpring Kipan collaboration

One maintainer works with local agents (Gemini and Claude in Antigravity, Claude Code on the maintainer's Mac) and cloud agents (Claude Code on the web, Jules) that reach the project through GitHub. These rules hold for all of them.

### Soft fork

- **Upstream**: `https://github.com/taigikeyboard/taigikeyboard`, branch `main`. Fork point `bd6900f` (v3.6.13). `#NNN` in commits up to the fork point refers to upstream's PRs.
- **Change user-facing identity**: app names, bundle IDs / `applicationId`, user-data directories and file names, icons, URLs, UI strings.
- **Keep upstream identifiers**: do not rename `taigi`, `tl`, `poj`, `tps`, `com.siansiansu.taigikeyboard`, crate / module / file names in code you do not otherwise need to change. Every rename turns an upstream fix into a merge conflict. Mass renames are out of scope unless the maintainer asks.
- **Hakfa code goes in new modules** named `hakfa` (e.g. `engine/phonetics/src/hakfa_tables.rs`, `dictionary/sources/official/hakspring/`). In upstream files, add the smallest hook that reaches the new module rather than rewriting the file.
- **Merging upstream**: `git fetch https://github.com/taigikeyboard/taigikeyboard.git main`, merge (never rebase) on a branch, regenerate dictionary binaries and i18n outputs with the repo's tooling instead of hand-resolving them, and commit as `chore(upstream): merge taigikeyboard main <short-sha>`.
- **Contributing back**: an engine improvement that helps Taigi too goes to upstream as its own PR, on a branch cut from upstream `main`.

### Where project knowledge lives

| Layer | What | Where |
|---|---|---|
| Code-bound knowledge | build pitfalls, architecture, data rules, how-tos | this repo, `docs/hakfa/` — commit it with the code it describes |
| Decisions, status, roadmap | what is decided, what is next | the maintainer's notes (below) |
| Discussion | maintainer ↔ agent threads | the maintainer's notes (below) |

**The maintainer's notes** are a private Obsidian vault, kept in a private GitHub repository of the maintainer's. Kipan's notes are under `創作/程式/HakSpring/IME/`:

- `IME.md` — the hub: current status, the decisions table, roadmap, active discussion. **Read it first** when you have the vault. A row in its decisions table overrides anything said in an archived thread.
- `dialects.md`, `android.md`, `ui-strings.md`, `names.md` — sub-plans.
- `log/` — archived threads (read-only; read them only to learn why something was decided).
- `hub-template.md` — the format every note follows.

Access: local agents on the maintainer's Mac have the vault; a session the maintainer explicitly attaches it to has it. Cloud sessions normally do not — do not ask for it. Work from the issue, which the maintainer writes to be self-contained.

Writing in the vault — only when asked, and following `hub-template.md`:

- Traditional Chinese (zh-TW), full-width punctuation, timestamps in UTC+8 as `<sup>YYYYMMDD HH:MM</sup>`.
- Your section is `### C<n>` (Claude) or `### G<n>` (Gemini) with a model tag line such as `#Claude/Opus-5-5/Code` under the heading. `### R<n>` sections belong to the maintainer; never write one.
- Append; never rewrite another author's section. Check whether a file exists before creating it.
- Obsidian links: `[[full/path|text]]`, embeds `![[full/path#heading]]`.
- When a decision is made, update its row in the hub's decisions table.

**Privacy**: the vault holds personal material outside `創作/程式/HakSpring/`. Never read, quote or copy anything outside that folder into this repository, an issue, a PR or a commit. Even from the Kipan notes, carry over technical content only — no personal remarks, no local paths such as `/Users/<name>/…` (also § Privacy in `docs/contributing/known-pitfalls.md`).

### Issues, PRs, commits

- **Open an issue** for work handed to a cloud agent, or for a user report. The body stands alone: goal, the decisions that apply (pasted, not linked to the vault), acceptance criteria, likely files.
- **One logical change per PR and per commit.** Do not bundle a platform, i18n, dictionary data and tooling into one commit.
- **Commit messages and PR descriptions are in Hakfa** — Hakfa Hanji（客語漢字）, the maintainer's decision for this fork.
    - **【鐵則】必須使用臺灣客家語（Hakfa）撰寫**：使用四縣／海陸等共通詞彙（如「个」、「摎」、「在」、「肚底／裡肚」、「單淨」、「無共樣」、「毋使」等）。嚴禁誤用臺灣話／閩南語詞彙（如「干焦」、「無仝」、「佮」、「佇」、「親像」等）或純華語長篇文言。保持客源翠專案文化自覺與語言一致性。
    - Keep the Conventional Commits prefix `type(scope):` in English and identifiers as they are in the code. The body says what changed, why, and which platforms were built and tested. Exceptions: a PR to upstream TaigiKeyboard is in English; an upstream merge commit keeps upstream's own messages.
- **Commit Message 審核與提交流程**：
    1. 先分析專案 Git 目前 staged 的 diff，歸納此次更新內容。
    2. 依對話記憶提及不同 AI 模型，分別列出每個 AI 模型的分工及貢獻。
    3. 先寫在專案根目錄的臨時 `commit_msg.txt` 中，供平哥審閱、直接編輯。
    4. Commit message **毋使寫來 Obsidian 筆記裡肚**（避免筆記內容過度冗長繁雜、重複堆疊已存在於 git log 或專案檔案的內容）。
    5. 平哥審核完成告知定稿後，再根據 `commit_msg.txt` 執行 commit。
        - 絕對要小心避免那種會卡住終端機狀態的指令。
        - 愛細膩莫 commit 著 `commit_msg.txt`；commit 煞後，暫存檔案愛提掉（清理）。
- **貢獻者樣板 (Co-authors Template)**（請依實際參與該次工作的 AI 角色與模型動態填寫，作者簽名一律使用 GitHub noreply 匿名信箱）：
```
Co-authored-by: <AI Model Name> <<model-id>@Antigravity>
Signed-off-by: GHSRobert Ciang <93833368+Aiuanyu@users.noreply.github.com> (Commit Helper: <Commit Helper Model> <<helper-id>@Antigravity>)
```
- After a PR merges, a local agent with the vault adds one line to the hub (what changed + PR URL).

### Reporting

- Open with three lines: what changed, the result, what the maintainer has to decide.
- Keep **verified** (a command you ran in this session and its output) apart from **inferred**. Never write "all tests pass" or "100 % done" for something you did not run — and name the stale-artifact gate (`make dict`, `make build`) you ran before a platform test.
- Hakfa phonetics: never carry a TL / POJ rule over to HPP by analogy (HPP has no `tsh` / `chh`; its stop finals are `b d g`). When unsure, ask the maintainer.

## Project Structure

```
android/  ios/  macos/  windows/  linux/   # platform apps
                   # macos/ links one Rust archive, macos/crates/taigi-macos-ffi (engine seam + taigi-desktop-core)
engine/            # Shared Rust engine — Cargo workspace; FFI to iOS / Android / macOS, linked in-process by Windows / Linux
desktop/           # Rust crates shared by the desktops — taigi-desktop-core (all three), -storage (Windows + Linux), -update (Windows)
dictionary/        # Dictionary sources + build pipeline + output artifacts
assets/            # Committed shipped data every platform packages — dictionaries/ (pipeline output), fonts/, symbols/
docs/              # hakfa/ (Kipan-specific), engine/, architecture/, contributing/, phonetics/ (TL/POJ/TPS reference), ui/, references/, reports/, roadmap.md
taigi-converter/   # Canonical TL↔POJ↔TPS converter (git submodule)
tools/             # Dev tooling — test selection, i18n codegen, release/, secret-scan/
e2e/               # End-to-end typing tests — scenarios/, analyzer/, drivers/
corpus/            # Real Taiwanese text — taigi-typing (manual-test sentences) + taigi-corpus (opt-in, future LM data); never a build input
changelog/         # Per-release changelogs — edit only at release time
references/        # Cloned external IME repos (gitignored)
emoji/             # Emoji data generator (own instructions file)
```

## Core Principles

1. **Xcode project files are maintainer-only** — agents never edit `.xcodeproj` / `.pbxproj` / `.xcworkspace`. Xcode synchronized groups auto-include new files under most `Sources/TaigiKeyboard/*` subdirs — exceptions in `docs/contributing/ios-guidelines.md`. Android Gradle files are editable.
2. **Cross-platform alignment** — align on **intended behavior**, not API calls; verify each platform independently; document when the same behavior needs different implementations (`docs/contributing/cross-platform-alignment.md`).
3. **Phonetics = authoritative-source-only** — never infer TL/POJ/TPS rules from test/dictionary absence; read `docs/phonetics/taigi-phonetics-reference.md` and `taigi-converter/` first (`docs/contributing/phonetics.md`). For Hakfa (HPP, the MOE Hakfa romanization, PFS) the sources are `engine/phonetics/src/hakfa_tables.rs`, `docs/hakfa/` and the maintainer; the Taigi references above govern the Taigi code paths only.
4. **Bugfix = root cause first** — reproduce the failure and pin the cause to `file:line` before writing the fix; the PR states both.
5. **Direction-first over fix-scope** — between two correct fixes prefer consistency and correct architectural direction over the smaller diff; state the trade-off.
6. **Word identity = (漢字, canonical-TL) pair** — neither alone is a key (`重/tîng` ≠ `重/tāng`). Governs every dedup / lookup / ranking-merge / variant decision project-wide. POJ/TPS are alternate renderings of the same TL reading. In Kipan the canonical reading is the HPP reading stored in the `tl` field.

## Design principles

- **No redundant fallback** — "A fails → fall back to B" is an anti-pattern for primary data flow (fetch, query, transform, dispatch). Push B's capability into A and keep the flow one-directional. Defensive guards (boundary checks, empty-state defaults) are not fallbacks; a temporary compatibility fallback says "temporary, remove after X" where it is introduced.
- **Shared logic lives in the engine** — behavior every platform needs is written once in Rust (`docs/contributing/rust-migration-policy.md`); platforms keep UI and OS integration.

## Read before…

| Before… | Read |
|---|---|
| your first change | § HakSpring Kipan collaboration above, then `docs/contributing/known-pitfalls.md` |
| any Hakfa phonetics, dictionary or dialect work | `docs/hakfa/`; with the vault, also the hub's decisions table and `dialects.md` |
| changing files on a platform | that platform's guide in `docs/contributing/` (index: `docs/contributing/README.md`) |
| adding / renaming / deleting an `i18n/*.json` key from platform code | `docs/contributing/i18n.md` |
| changing any UI string in Kipan | `docs/hakfa/i18n-overlay.md` — Kipan strings live in `i18n/hakfa/`; never edit `i18n/*.json` |
| adding or changing a framework / OS API call (KeyboardKit, `UITextDocumentProxy`, `InputMethodService`, Compose, DataStore) | `docs/contributing/doc-lookup.md` — verify against current docs, never from memory |
| a real-device dogfood pass, or citing an `Sn` item | `docs/architecture/dogfood-checklist.md`; draw test sentences from `corpus/README.md` |
| a best-practices section, or any segmentation / lattice / ranking / next-word / continuous-input design | `docs/references/mainstream-ime-comparison.md` — never re-explore `references/` from scratch |

## Build & Test

Full setup: [`docs/BUILDING.md`](docs/BUILDING.md). **Bootstrap**: clone with `--recurse-submodules`, then `make build` once per machine — generates the xcframeworks and `jniLibs/*.so` the app builds link (not committed) and regenerates the committed platform protos.

**Test what you touched**: `python3 tools/test_select.py --run --platform <p>` per touched platform (no args = print the selection; `make test-changed` runs the whole selection). The full suites below are for release prep or a deliberate catch-net. `make lint` runs every formatter / linter gate CI enforces.

| Platform | Build | Test |
|---|---|---|
| engine | `cargo build --manifest-path engine/Cargo.toml --workspace` | `make test` |
| iOS | Xcode → keyboard extension | `xcodebuild -project ios/TaigiKeyboard.xcodeproj -scheme TaigiKeyboardTests -destination 'platform=iOS Simulator,name=<device>' test` |
| Android | `android/gradlew -p android :app:assembleDebug` | `android/gradlew -p android :app:testDebugUnitTest` |
| macOS | `make -C macos build` (`install` before dogfood) | `make -C macos test` |
| Windows | `make windows-check` (host gate; its `check-box` step needs the maintainer's Windows box over ssh — others run the sub-targets in `windows/README.md`; the DLL builds only on Windows — `docs/architecture/windows-release.md`) | included |
| Linux | `make linux-check` (host gate; `.deb` via `make -C linux deb` — `docs/architecture/linux-release.md`) | included |
| desktop-shared / macOS Rust crates | `make desktop-check` (`desktop/`) · `make macos-rust-check` (`macos/crates`) | included |
| taigi-converter | — | `npm test` in `taigi-converter/` |

**Stale-artifact gate — run before every iOS / Android / macOS build+test** (stale binaries give false-green tests):

| Diff touches… | Run first |
|---|---|
| `engine/` (Rust, `Cargo.toml`) | `make build` (or the one platform's `make ios-libs` / `android-libs` / `macos-libs`) |
| a `.proto` file | `make build` (or `make protos`, then the platform's target) |
| `dictionary/` | `make dict` then `make build` |
| `desktop/`, `macos/crates/`, `macos/Cargo.*` (the archive macOS links) | `make macos-libs` (or `make build`) before a macOS build or test |
| platform-only Swift / Kotlin / docs | nothing |

Canonical: `docs/BUILDING.md` §4–5; rationale and timings: `docs/architecture/build-artifacts.md`.

## Conventions

- **【族名與語名規範】一律使用 `hakfa`（Hakfa）／嚴禁使用 `hakka`**：專案全體（包含程式碼命名、變數、檔名、路徑、文件與 UI），若涉及客語羅馬化代號或族語命名，**嚴禁使用 ❌ `hakka`，一律使用 ⭕ `hakfa`（有大寫時就大寫，如 `Hakfa`）**。原因：臺灣客人大體喊自家 Hagngin、喊族語 Hakfa。
- Code, comments and docs in **English** (Kipan exception: commit messages and PR descriptions are in Hakfa — § Issues, PRs, commits). Living docs name UI labels by their i18n `en` value. CJK only for Taigi content (phonetic / Hanji examples, test data, proper names of dictionaries and fonts). Dated snapshots (`docs/reports/**`, `docs/releases/**`) are frozen.
- Conventional Commits `type(scope): subject`; one PR per logical change; say which platforms you built and tested.
- `changelog/` is edited only at release time.
- No personal identifiers in tracked files, commits or PRs (`CONTRIBUTING.md` § Secrets).

## References

- `docs/architecture/system-overview.md` — read first for architecture
- `docs/README.md` — documentation index · `docs/roadmap.md` — live multi-PR plans
- `#NNN` written before 2026-09-07 = old repository; resolve via `git log --all --oneline --grep="(#NNN)"` (`docs/architecture/pr-number-migration.md`)
