"""Regression for the HakSpring certification 【】 variants in `hakspring.csv`.

The certification Sixian lists write two different things inside 【】:

* a synonym of the South Sixian form: 好額【有口（有發）】 / iú hèu (iú bòd) — both are South forms;
* a Meinong reading that belongs to the South word: 佢兜人【佢等】 ǐ dén (ǐ nén) — `ǐ nén` is 佢等's.

`js/variant-parser.js` used to read every `X (Y)` as "South X, Meinong Y" and hang Meinong on the
North word, so 叔姆 got `mí-ngiá` (that is 嬭若), 有發 got `iú-hèu`, 救雨 got `biang-ì` and 寄雨 got
`giu-ì`. These tests pin the corrected pairs in the shipped data.
"""

from __future__ import annotations

import csv
from pathlib import Path

DATA = Path(__file__).resolve().parents[1] / "sources/official/hakspring/data/hakspring.csv"


def _rows(lect: str) -> dict[str, set[str]]:
    out: dict[str, set[str]] = {}
    with DATA.open(encoding="utf-8", newline="") as f:
        for r in csv.DictReader(f):
            if r["lect"] == lect:
                out.setdefault(r["word"], set()).add(r["tl"])
    return out


def test_south_synonyms_keep_their_own_readings():
    south = _rows("南")
    assert "mí-ngiá" in south["嬭若"], south["嬭若"]
    assert "mí-ngiá" not in south["叔姆"], south["叔姆"]  # 嬭若's reading, not 叔姆's
    assert "iú-bòd" in south["有發"], south["有發"]
    assert "iú-hèu" in south["有口"], south["有口"]
    assert "giu-ì" in south["救雨"], south["救雨"]
    assert "giu-ì" not in south["寄雨"], south["寄雨"]  # 救雨's reading, not 寄雨's
    assert "iú-bòd" not in south["好額"], south["好額"]


def test_meinong_reading_belongs_to_the_south_word():
    south = _rows("南")
    assert "ǐ-nén" in south["佢等"], south["佢等"]
    assert "ǐ-nén" not in south["佢兜人"], south.get("佢兜人")


def test_xiu_is_not_a_single_character_reading():
    # 壽 xiǔ only occurs inside 夭壽 iáu xiǔ; the word itself stays.
    south = _rows("南")
    assert "xiǔ" not in south.get("壽", set()), south.get("壽")
    assert "iáu-xiǔ" in south["夭壽"], south["夭壽"]
