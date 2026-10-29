# Feature x platform matrix

Which platform's source mentions which Hakfa feature. It exists so a feature that only one platform has (the 各腔互學 entry is macOS-only) is visible at a glance, not found by accident.

**How to read it.** A cell is the number of hand-written source files under that platform's directories that match the feature's pattern (`tools/hakfa/platform-matrix.py`; generated protobuf code, build output and tests are skipped). `-` means none. It says the code *mentions* the feature, **not that it works on a device**: read the files before trusting a cell, and dogfood before claiming a platform. `desktop` is the shared Rust crate macOS, Windows and Linux link; a shell's own cell counts only code in that shell.

**Keeping it current.** When a change adds or removes a feature on a platform, run `python3 tools/hakfa/platform-matrix.py --write` and commit the refreshed table with it. `python3 tools/hakfa/platform-matrix.py --check` fails when the table is stale. To add a feature, add a row to `FEATURES` in the script. Per-lect data counts live in the maintainer's notes (the lect matrix), not here.

<!-- matrix:begin -->
| 功能 | engine | desktop | iOS | Android | macOS | Windows | Linux |
|---|---:|---:|---:|---:|---:|---:|---:|
| 各腔互學 (sync lects) 入口 | 2 | 1 | - | - | 3 | - | - |
| 學習紀錄頁 (learning records) | 6 | 6 | 6 | 11 | 4 | 3 | 4 |
| 數字調打法 (digit tones) | - | 11 | - | 1 | 6 | 2 | 1 |
| 備份匯出入 (backup) | 3 | - | 5 | 5 | - | - | - |
| 符號面板 (symbol panel) | - | 7 | 2 | 2 | 10 | 3 | 3 |
| 腔別 (lects) | - | 6 | - | - | 2 | - | - |
<!-- matrix:end -->
