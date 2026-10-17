#!/usr/bin/env python3
# Kipan's Hakfa layer over the upstream i18n sources (docs/hakfa/i18n-overlay.md).
#
# i18n/*.json stay byte-identical to upstream TaigiKeyboard so an upstream merge never conflicts on
# them. Kipan's own strings live in i18n/hakfa/<namespace>.json, one line per key:
#
#     "reset": { "hakha": "恢復" },
#     "appHeaderTitle": { "hakha": "客源翠企盤", "en": "HakSpring-Kipan", ... },
#
# apply() runs on every namespace before validation: upstream's `hanji` (Taigi Hanji) becomes Kipan's
# `taiha`, then the overlay values replace or add per language. Every upstream key needs an overlay
# entry, because `hakha` is the base language; `--seed` writes the missing ones for review.

import json
import sys
from pathlib import Path

OVERLAY_DIR = "i18n/hakfa"
UPSTREAM_TAIGI_HANJI = "hanji"
KIPAN_TAIGI_HANJI = "taiha"


def _overlay_path(repo_root: Path, namespace: str) -> Path:
    return repo_root / OVERLAY_DIR / f"{namespace}.json"


def _load(path: Path) -> dict:
    from i18n_lib import load_namespace

    return load_namespace(path)


def apply(namespace: str, data: dict, repo_root: Path) -> dict:
    keys = data.get("keys")
    if not isinstance(keys, dict):
        return data
    for key, entry in keys.items():
        values = entry.get("values", {})
        if UPSTREAM_TAIGI_HANJI in values:
            if KIPAN_TAIGI_HANJI in values:
                raise ValueError(f"{namespace}:{key}: has both {UPSTREAM_TAIGI_HANJI!r} and {KIPAN_TAIGI_HANJI!r}")
            entry["values"] = {
                (KIPAN_TAIGI_HANJI if lang == UPSTREAM_TAIGI_HANJI else lang): text for lang, text in values.items()
            }
    path = _overlay_path(repo_root, namespace)
    if not path.exists():
        return data
    overlay = _load(path)
    if overlay.get("namespace") != namespace:
        raise ValueError(f"{path}: top-level 'namespace' must equal {namespace!r}")
    for key, values in overlay.get("keys", {}).items():
        if key not in keys:
            raise ValueError(f"{path}:{key}: no such key in i18n/{namespace}.json — upstream removed or renamed it")
        if not isinstance(values, dict) or not values:
            raise ValueError(f"{path}:{key}: must be a non-empty object of language -> text")
        # Overlay languages first so `hakha` (the base) leads every merged values object.
        keys[key]["values"] = {**values, **{lang: text for lang, text in keys[key]["values"].items() if lang not in values}}
    return data


def check_orphans(repo_root: Path) -> None:
    for path in sorted((repo_root / OVERLAY_DIR).glob("*.json")):
        if not (repo_root / "i18n" / path.name).exists():
            raise ValueError(f"{path}: no i18n/{path.name} for this overlay")


def missing_hakha(repo_root: Path) -> dict:
    # namespace -> [(key, taiha text)] for upstream keys that have no `hakha` yet.
    from i18n_lib import BASE_LANGUAGE

    missing = {}
    for base in sorted((repo_root / "i18n").glob("*.json")):
        namespace = base.stem
        data = apply(namespace, _load(base), repo_root)
        for key, entry in data["keys"].items():
            if BASE_LANGUAGE not in entry["values"]:
                missing.setdefault(namespace, []).append((key, entry["values"].get(KIPAN_TAIGI_HANJI, "")))
    return missing


def seed(repo_root: Path) -> int:
    # Appends `"key": { "hakha": <taiha text> }` for every missing key, as a starting point a person
    # rewrites into Hakfa. Existing lines are left byte-for-byte as they are.
    missing = missing_hakha(repo_root)
    for namespace, pairs in missing.items():
        path = _overlay_path(repo_root, namespace)
        lines = [json.dumps(k, ensure_ascii=False) + ": " + '{ "hakha": ' + json.dumps(v, ensure_ascii=False) + " }" for k, v in pairs]
        if path.exists():
            text = path.read_text(encoding="utf-8").rstrip()
            head, tail = text.rsplit("}", 2)[0], "  }\n}\n"
            head = head.rstrip()
            sep = "," if not head.endswith("{") else ""
            body = head + sep + "\n" + ",\n".join("    " + line for line in lines) + "\n" + tail
        else:
            body = '{\n  "namespace": "' + namespace + '",\n  "keys": {\n' + ",\n".join("    " + line for line in lines) + "\n  }\n}\n"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body, encoding="utf-8")
        for key, _ in pairs:
            print(f"hakfa-overlay: seeded {namespace}:{key} — review the Hakfa text")
    return sum(len(p) for p in missing.values())


def main(argv) -> int:
    repo_root = Path(__file__).resolve().parents[2]
    if argv[1:] == ["--seed"]:
        seed(repo_root)
        return 0
    missing = missing_hakha(repo_root)
    for namespace, pairs in missing.items():
        for key, _ in pairs:
            print(f"hakfa-overlay: {namespace}:{key} has no hakha value", file=sys.stderr)
    if missing:
        print("hakfa-overlay: run `python3 tools/i18n/hakfa_overlay.py --seed`, then rewrite the seeded lines", file=sys.stderr)
        return 1
    print("hakfa-overlay: every key has a hakha value")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
