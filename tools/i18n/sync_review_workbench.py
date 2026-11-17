#!/usr/bin/env python3
"""Sync an HTML review workbench's `rawData` array with i18n/*.json and i18n/hakfa/*.json.

Usage:
    python3 tools/i18n/sync_review_workbench.py <path/to/workbench.html>
"""

import glob
import json
import os
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
I18N_DIR = REPO_ROOT / "i18n"
OVERLAY_DIR = I18N_DIR / "hakfa"


def load_entries() -> list[dict]:
    base_map: dict[tuple[str, str], dict] = {}
    ordered_keys: list[tuple[str, str]] = []

    for path in sorted(glob.glob(str(I18N_DIR / "*.json"))):
        with open(path, encoding="utf-8") as f:
            doc = json.load(f)
        ns = doc["namespace"]
        for key, meta in doc.get("keys", {}).items():
            vals = meta.get("values", {})
            pair = (ns, key)
            ordered_keys.append(pair)
            base_map[pair] = {
                "ns": ns,
                "key": key,
                "en": vals.get("en", ""),
                "hanji": vals.get("hanji", ""),
                "hakha": "",
                "comment": meta.get("comment", ""),
            }

    for path in sorted(glob.glob(str(OVERLAY_DIR / "*.json"))):
        with open(path, encoding="utf-8") as f:
            doc = json.load(f)
        ns = doc["namespace"]
        for key, meta in doc.get("keys", {}).items():
            pair = (ns, key)
            if "values" in meta:
                vals = meta["values"]
                hakha = vals.get("hakha", "")
                if pair not in base_map:
                    ordered_keys.append(pair)
                    base_map[pair] = {
                        "ns": ns,
                        "key": key,
                        "en": vals.get("en", ""),
                        "hanji": vals.get("taiha", hakha),
                        "hakha": hakha,
                        "comment": meta.get("comment", ""),
                    }
                else:
                    base_map[pair]["hakha"] = hakha
            else:
                if pair in base_map and "hakha" in meta:
                    base_map[pair]["hakha"] = meta["hakha"]

    return [base_map[pair] for pair in ordered_keys]


def main() -> int:
    if len(sys.argv) != 2:
        print("Usage: sync_review_workbench.py <path/to/workbench.html>", file=sys.stderr)
        return 2

    html_path = Path(sys.argv[1]).expanduser().resolve()
    if not html_path.is_file():
        print(f"File not found: {html_path}", file=sys.stderr)
        return 1

    html = html_path.read_text(encoding="utf-8")
    pattern = re.compile(r"(const\s+rawData\s*=\s*)(\[.*?\])(;)", re.S)
    match = pattern.search(html)
    if not match:
        print("Could not locate `const rawData = [...];` in HTML.", file=sys.stderr)
        return 1

    old_data = json.loads(match.group(2))
    new_data = load_entries()
    serialized = json.dumps(new_data, ensure_ascii=False)
    updated_html = html[: match.start(2)] + serialized + html[match.end(2) :]
    html_path.write_text(updated_html, encoding="utf-8")

    print(f"Updated {html_path.name}: {len(old_data)} -> {len(new_data)} entries.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
