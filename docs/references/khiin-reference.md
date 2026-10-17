# khiin-rs Reference Research

> **Type**: Reference
> **Keywords**: `khiin`, `起引`, `DPSegment`, `Bigram`, `Trie`
> **Related**: rime-reference.md, mainstream-ime-comparison.md (card #12)

Facts about `references/khiin-rs/` (Rust Taigi IME for desktop). What TaigiKeyboard took from it: the walker cost in `engine/composing/src/lattice/cost.rs` ports the segmenter below (`walker.rs` cites `segment_min_cost`).

---

## Architecture

### Two tries

| Trie | File | Use | Methods |
|------|------|-----|---------|
| Word trie | `khiin/src/data/trie.rs` | Key sequence → word IDs | `find_words_by_prefix`, `find_words_from_start`, `contains` |
| Syllable trie | `khiin/src/data/syllable_trie.rs` | Syllable validation | `is_valid_prefix`, `is_valid_syllable` |

### Database (`khiin/src/db/migrations/001/up.sql`)

- `unigrams(gram, n)` — candidate order is `u.n desc, c.weight desc` (`khiin/src/db/sql/select_conversions.sql`).
- `bigrams(lgram, rgram, n)` — **schema only**: the bigram join in `select_conversions.sql` is commented out and nothing in `khiin/src/` writes n-gram counts.

### Config (`khiin/src/config/conf.rs`)

| Setting | Values |
|---------|--------|
| `InputMode` | Continuous / Classic / Manual |
| `ToneMode` | Numeric / Telex (default keys: t2 `s`, t3 `f`, t5 `l`, t6 `x`, t7 `j`, t8 `j`, t9 `w`, khin `v`) |
| `KhinMode` (neutral tone) | Khinless / Hyphen / Dot |

### Input parsing (`khiin/src/input/parser.rs`)

`SectionType ∈ {Plaintext, Hyphens, Punct, Splittable}`; unknown input passes through verbatim as `Plaintext`. There is no fuzzy / typo-correction tier anywhere in `khiin/src/`.

---

## DP segmentation (`khiin/src/data/segmenter.rs`)

Word-level DP: edges are dictionary words, each priced by

```rust
const FREQUENCY_BIAS: f64 = 1.0;       // weight of word frequency
const LETTER_COUNT_BIAS: f64 = 0.2;    // preference for longer words
const SYLLABLE_COUNT_BIAS: f64 = 0.2;  // preference for fewer syllables

cost = ln(1 / p^FREQUENCY_BIAS) / word_len^LETTER_COUNT_BIAS * n_syls^SYLLABLE_COUNT_BIAS
// p <= 0 → p = 1e-5 / 10^word_len
```

`segment_min_cost` minimizes the summed cost. Higher frequency and longer spelling lower the cost; more syllables raise it slightly.

---

## Syllables and tones (`ji/src/lomaji.rs`)

- Tone-mark placement (`TONE_LETTER_PATTERNS`): `o[ae]` + final → the second vowel, then `o > a > e > u > i > ng > n > m`.
- Legal-syllable regex `RE_LEGAL_LOMAJI`:
  `^·?((chh|[ckpt]h|[bhgjklmnpst])?(iau|io͘|oai|a[iu]|i[aou]|o[ae͘]|ui|[aeiou])?(ng|[mnptkh])?|(chh|[ckpt]h|[hkmnpst])ng?|(ng|m)h?)(n|ⁿ|ᴺ)?$`
- `o͘` expands to the key sequences `oo` and `ou`.

---

## Source map

| File | Content |
|------|---------|
| `ji/src/lomaji.rs` | Tone placement, syllable validation |
| `ji/src/tone.rs` | Tone enum and conversion |
| `khiin/src/data/segmenter.rs` | DP segmentation |
| `khiin/src/input/converter.rs` | Candidate generation |
| `khiin/src/buffer/buffer_mgr.rs` | Buffer management (commit-and-resegment) |
