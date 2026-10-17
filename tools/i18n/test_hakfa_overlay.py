#!/usr/bin/env python3
# Unit tests for the Kipan Hakfa i18n overlay (hakfa_overlay.py). Run via `make i18n-test`.

import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import hakfa_overlay  # noqa: E402
from i18n_lib import load_namespace  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[2]
SCOPE = {"platforms": ["android"], "surfaces": ["host"]}


def _base(keys: dict) -> dict:
    return {"namespace": "ns", "keys": {k: {"scope": SCOPE, "values": v} for k, v in keys.items()}}


class OverlayTest(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        (self.root / "i18n" / "hakfa").mkdir(parents=True)

    def tearDown(self):
        self._tmp.cleanup()

    def _write(self, rel: str, data: dict) -> None:
        (self.root / rel).write_text(json.dumps(data, ensure_ascii=False), encoding="utf-8")

    def test_upstream_hanji_becomes_taiha(self):
        data = hakfa_overlay.apply("ns", _base({"k": {"hanji": "齒盤", "en": "Keyboard"}}), self.root)
        self.assertEqual(data["keys"]["k"]["values"], {"taiha": "齒盤", "en": "Keyboard"})

    def test_overlay_adds_hakha_first_and_replaces(self):
        self._write("i18n/hakfa/ns.json", {"namespace": "ns", "keys": {"k": {"hakha": "企盤", "en": "Kipan"}}})
        data = hakfa_overlay.apply("ns", _base({"k": {"hanji": "齒盤", "en": "Keyboard"}}), self.root)
        values = data["keys"]["k"]["values"]
        self.assertEqual(values, {"hakha": "企盤", "en": "Kipan", "taiha": "齒盤"})
        self.assertEqual(next(iter(values)), "hakha")

    def test_overlay_key_missing_upstream_rejected(self):
        self._write("i18n/hakfa/ns.json", {"namespace": "ns", "keys": {"gone": {"hakha": "無"}}})
        with self.assertRaisesRegex(ValueError, "upstream removed or renamed"):
            hakfa_overlay.apply("ns", _base({"k": {"hanji": "字"}}), self.root)

    def test_overlay_namespace_mismatch_rejected(self):
        self._write("i18n/hakfa/ns.json", {"namespace": "other", "keys": {}})
        with self.assertRaisesRegex(ValueError, "namespace"):
            hakfa_overlay.apply("ns", _base({"k": {"hanji": "字"}}), self.root)

    def test_both_hanji_and_taiha_rejected(self):
        with self.assertRaisesRegex(ValueError, "both"):
            hakfa_overlay.apply("ns", _base({"k": {"hanji": "字", "taiha": "字"}}), self.root)

    def test_orphan_overlay_rejected(self):
        self._write("i18n/hakfa/ns.json", {"namespace": "ns", "keys": {}})
        with self.assertRaisesRegex(ValueError, "no i18n/ns.json"):
            hakfa_overlay.check_orphans(self.root)

    def test_seed_appends_missing_keys_and_keeps_existing_lines(self):
        self._write("i18n/ns.json", _base({"a": {"hanji": "甲"}, "b": {"hanji": "乙"}}))
        existing = '{\n  "namespace": "ns",\n  "keys": {\n    "a": { "hakha": "甲客" }\n  }\n}\n'
        (self.root / "i18n/hakfa/ns.json").write_text(existing, encoding="utf-8")
        self.assertEqual(hakfa_overlay.missing_hakha(self.root), {"ns": [("b", "乙")]})
        self.assertEqual(hakfa_overlay.seed(self.root), 1)
        text = (self.root / "i18n/hakfa/ns.json").read_text(encoding="utf-8")
        self.assertIn('    "a": { "hakha": "甲客" },\n', text)
        self.assertEqual(json.loads(text)["keys"]["b"], {"hakha": "乙"})
        self.assertEqual(hakfa_overlay.missing_hakha(self.root), {})


class ProductionOverlayTest(unittest.TestCase):
    # The point of the overlay: upstream's files stay upstream-shaped, Kipan's strings stay complete.
    def test_upstream_sources_carry_no_kipan_languages(self):
        for path in sorted((REPO_ROOT / "i18n").glob("*.json")):
            for key, entry in load_namespace(path)["keys"].items():
                langs = set(entry["values"])
                self.assertFalse(langs & {"hakha", "taiha"}, f"{path.name}:{key} carries a Kipan language — move it to i18n/hakfa/")

    def test_every_key_has_hakha(self):
        self.assertEqual(hakfa_overlay.missing_hakha(REPO_ROOT), {})


if __name__ == "__main__":
    unittest.main()
