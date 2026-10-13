# librime Reference Research

> **Type**: Reference
> **Keywords**: `librime`, `RIME`, `Pipeline`, `DAG`, `SpellingAlgebra`
> **Related**: khiin-reference.md, mainstream-ime-comparison.md (card #8)

---

## Summary

- Modular Pipeline architecture: Processor → Segmentor → Translator → Filter
- DAG segmentation algorithm for continuous input
- Spelling Algebra for rule-based spelling variants
- Dynamic weight + time decay for user learning
- YAML configuration-driven design

---

## Architecture Overview

### Pipeline Processing Flow

```
Key input
    ↓
Processors   Handle key events (Speller, Selector, Punctuator)
    ↓
Segmentors   Segment input (AbcSegmentor, PunctSegmentor)
    ↓
Translators  Generate candidates (ScriptTranslator, TableTranslator)
    ↓
Filters      Filter/sort (Uniquifier, Simplifier)
    ↓
Candidate list
```

### Core Classes

| Class | Responsibility |
|-------|----------------|
| Engine | Orchestrator, manages Pipeline components |
| Context | Context, stores input, candidates, state |
| Schema | Configuration, defines IME behavior |

---

## User Dictionary and Sorting

### Data Structure

```cpp
struct UserDbValue {
    int commits = 0;      // Commit count
    double dee = 0.0;     // Dynamic weight factor
    TickCount tick = 0;   // Timestamp
};
```

### Dynamic Weight Algorithm

```cpp
// Time decay formula
inline double formula_d(double d, double t, double da, double ta) {
    return d + da * exp((ta - t) / 200);
}
```

### Candidate Sorting

| Priority | Description |
|----------|-------------|
| 1 | Exact match > Predictive match |
| 2 | User vocabulary +0.5 bonus |
| 3 | Complete input > Autocomplete (-1 penalty; both in `src/rime/gear/table_translator.cc:91`) |
| 4 | Sort by quality value |

### Storage

- LevelDB key-value database
- Key: `{code}\t{phrase}`
- Value: `c={commits} d={dee} t={tick}`

---

## Configuration System

### Schema Structure

```yaml
schema:
  schema_id: luna_pinyin
  name: Luna Pinyin

engine:
  processors: [speller, selector, punctuator]
  segmentors: [abc_segmentor, punct_segmentor]
  translators: [script_translator]
  filters: [uniquifier]

speller:
  alphabet: zyxwvutsrqponmlkjihgfedcba
  algebra:
    - derive/^([nl])ve$/$1ue/
```

### Configuration Directives

| Directive | Function |
|-----------|----------|
| `__include` | Include other configs |
| `__patch` | Patch configs |
| `__append` | Append to list |
| `__merge` | Merge tree structures |

### User Customization

`.custom.yaml` files override original configs:

```yaml
# luna_pinyin.custom.yaml
patch:
  menu/page_size: 9
  speller/algebra/+:
    - derive/^([jqxy])u/$1v/
```

---

## Spelling and Segmentation

### Spelling Algebra

| Type | Syntax | Function | Penalty |
|------|--------|----------|---------|
| `xform` | `xform/pattern/replacement/` | Regex transform | None |
| `derive` | `derive/pattern/replacement/` | Derive variant | None |
| `fuzz` | `fuzz/pattern/replacement/` | Fuzzy phonetics | -0.693 |
| `abbrev` | `abbrev/pattern/replacement/` | Abbreviation | -0.693 |
| `erase` | `erase/pattern/` | Delete | - |

### DAG Segmentation

Segmentation graph for input `xian`:

```
Position:  0 ──── 1 ──── 2 ──── 3 ──── 4
           └──xi──┴──an──┘
           └─────────xian──────────────┘
```

Possible splits: `xi + an` (Xi'an) or `xian` (先)

### Penalty Values

| Type | Penalty |
|------|---------|
| Abbreviation/Fuzzy | -0.693 = log(0.5) (`src/rime/algo/calculus.cc:14-15`) |
| Completion (autocomplete) | -2.996 = log(0.05) (`src/rime/algo/syllabifier.cc:27`) |
| Error correction | -4.605 = log(0.01) (`syllabifier.cc:28`, `calculus.cc:16`) |
| Ambiguous syllable | -2.996 (`src/rime/dict/table.cc:569`) |

---

## Other Features

| Feature | Description |
|---------|-------------|
| OpenCC | Simplified/Traditional conversion |
| Reverse lookup | Query one IME using another |
| Punctuation | Configuration-driven punctuation |
| Chord | Multiple keys pressed simultaneously |
| Sentence | Poet + Grammar for best sentence |
| History | Record recent input |

---

## librime Source Code Reference

| Directory | Content |
|-----------|---------|
| `src/rime/algo/` | Algorithms (dynamics, syllabifier, algebra) |
| `src/rime/dict/` | Dictionary (user_dictionary, prism, table) |
| `src/rime/gear/` | Components (processors, translators, filters) |
| `src/rime/config/` | Configuration (config_compiler, config_types) |
