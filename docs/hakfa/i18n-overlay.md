# Hakfa i18n overlay

Kipan keeps upstream's UI string sources untouched and layers its own strings on top, so merging upstream TaigiKeyboard never conflicts on `i18n/*.json`.

## Files

| Path | Owner | Content |
|---|---|---|
| `i18n/*.json` | upstream | Byte-identical to TaigiKeyboard. Languages `hanji` (Taigi Hanji), `tailo`, `poj`, `en`, `ja`. Never edit in Kipan. |
| `i18n/hakfa/<namespace>.json` | Kipan | One line per upstream key: `"key": { "hakha": "…" }`, plus any other language Kipan overrides (e.g. the app name in `home.appHeaderTitle`). |
| `tools/i18n/hakfa_overlay.py` | Kipan | The merge, the checks, and `--seed`. |
| `i18n/content/*.json` | upstream | FAQ / feature tour, decoded by the platforms directly; not covered by the overlay yet. |

## How the merge works

`tools/i18n/i18n_lib.py` calls `hakfa_overlay.apply()` on each namespace before validation:

1. Upstream's `hanji` is renamed to Kipan's `taiha` (Taigi Hanji kept as a display language).
2. The overlay's values replace or add per language; `hakha` (Hakfa Hanji, the base language) comes first.

Everything downstream — validation, `make i18n`, `tools/i18n/check.py`, the Gradle and macOS staleness gates — sees the merged result, so the generated files are what they were before the overlay existed.

The merge fails loudly when:

- an overlay key no longer exists upstream (upstream removed or renamed it — move or drop the line);
- an overlay file has no upstream namespace;
- an upstream key has no `hakha` (`make i18n` reports it as an incomplete production language).

## After an upstream merge

```sh
python3 tools/i18n/hakfa_overlay.py          # lists upstream keys without hakha
python3 tools/i18n/hakfa_overlay.py --seed   # appends them with the taiha text as a placeholder
# rewrite each seeded line into Hakfa, then:
make i18n && make i18n-test
```

A seeded line is Taigi text standing in for Hakfa. Rewrite it before release; `--seed` exists so a merge is not blocked on translation, not to ship Taigi strings.

## Adding a Kipan-only key

Not supported yet: every key must exist upstream. If Kipan needs one, add support for overlay-only keys in `hakfa_overlay.py` (with `scope`) rather than adding the key to `i18n/*.json`.
