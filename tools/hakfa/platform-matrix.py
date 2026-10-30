#!/usr/bin/env python3
"""Feature x platform matrix, measured from the source tree (Kipan).

    python3 tools/hakfa/platform-matrix.py            # print the markdown table
    python3 tools/hakfa/platform-matrix.py --check    # exit 1 when docs/hakfa/platform-matrix.md is stale

A cell is the number of hand-written source files under that platform's directories whose text
matches the feature's pattern (generated protobuf code, build output and tests are skipped).
"-" means none. It says the code MENTIONS the feature, never that the feature works on a device;
the shell of a shared desktop crate counts once under `desktop` and once more under each shell
that names it. Read the files before trusting a cell, and dogfood before claiming a platform.
"""
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOC = ROOT / "docs/hakfa/platform-matrix.md"

# column -> directories (relative to the repo root)
PLATFORMS = {
    "engine": ["engine"],
    "desktop": ["desktop"],
    "iOS": ["ios/Sources"],
    "Android": ["android/app/src/main"],
    "macOS": ["macos/Sources", "macos/crates"],
    "Windows": ["windows"],
    "Linux": ["linux"],
}
EXTENSIONS = {".rs", ".swift", ".kt", ".java"}
SKIP_PARTS = {"target", "build", "Generated", "proto", "tests", "androidTest", "test", "TaigiKeyboardTests"}
SKIP_NAMES = ("pb.swift",)

# feature -> (what the pattern means, regex)
FEATURES = {
    "各腔互學 (sync lects) 入口": (
        "a caller of the SyncLects request",
        r"SyncLects|syncLects|sync_lects|userDataSyncLects",
    ),
    "學習紀錄頁 (learning records)": (
        "the Learning Records page / its data",
        r"LearningRecord|learning_records",
    ),
    "數字調打法 (digit tones)": (
        "tone-digit scheme, Ctrl+digit, or the number-key hints",
        r"ToneDigit|apply_tone_digit|ToneInputScheme|toneHintForCode",
    ),
    "備份匯出入 (backup)": (
        "the user-data backup file",
        r"export_backup|import_backup|exportBackup|importBackup|BackupDocument",
    ),
    "符號面板 (symbol panel)": (
        "the punctuation / symbol picker",
        r"SymbolPicker|SymbolTable|symbol_picker|SymbolData|halfWidthRows",
    ),
    "腔別切換 (lect switching)": (
        "the lect choice (Xiien / Namxiian / Hoiliug) or the Android subtype's lect",
        r"Namxiian|Hoiliug|Xiien|CurrentLect",
    ),
}


def source_files(directory: Path):
    for path in directory.rglob("*"):
        if not path.is_file() or path.suffix not in EXTENSIONS:
            continue
        if any(part in SKIP_PARTS for part in path.parts) or path.name.endswith(SKIP_NAMES):
            continue
        yield path


def measure():
    cache = {}
    for column, directories in PLATFORMS.items():
        files = []
        for directory in directories:
            base = ROOT / directory
            if base.is_dir():
                files.extend(source_files(base))
        cache[column] = [(f, f.read_text(encoding="utf-8", errors="ignore")) for f in files]
    table = {}
    for feature, (_, pattern) in FEATURES.items():
        regex = re.compile(pattern)
        table[feature] = {
            column: sum(1 for _, text in texts if regex.search(text))
            for column, texts in cache.items()
        }
    return table


def render(table) -> str:
    columns = list(PLATFORMS)
    lines = ["| 功能 | " + " | ".join(columns) + " |", "|---|" + "---:|" * len(columns)]
    for feature, counts in table.items():
        cells = [str(counts[c]) if counts[c] else "-" for c in columns]
        lines.append(f"| {feature} | " + " | ".join(cells) + " |")
    return "\n".join(lines)


BEGIN, END = "<!-- matrix:begin -->", "<!-- matrix:end -->"


def current_block() -> str:
    text = DOC.read_text(encoding="utf-8") if DOC.exists() else ""
    if BEGIN in text and END in text:
        return text.split(BEGIN, 1)[1].split(END, 1)[0].strip()
    return ""


if __name__ == "__main__":
    rendered = render(measure())
    if "--check" in sys.argv:
        if current_block() != rendered:
            print("docs/hakfa/platform-matrix.md is stale; run: python3 tools/hakfa/platform-matrix.py --write")
            sys.exit(1)
        print("platform matrix is current")
    elif "--write" in sys.argv:
        text = DOC.read_text(encoding="utf-8")
        head, rest = text.split(BEGIN, 1)
        tail = rest.split(END, 1)[1]
        sha = subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT, capture_output=True, text=True).stdout.strip()
        DOC.write_text(f"{head}{BEGIN}\n{rendered}\n{END}{tail}", encoding="utf-8")
        print(f"wrote {DOC.relative_to(ROOT)} (measured at {sha})")
    else:
        print(rendered)
