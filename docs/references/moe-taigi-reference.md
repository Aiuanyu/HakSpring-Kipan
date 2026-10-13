# MOE Taigi IME Reference Analysis

> **Type**: Reference
> **Keywords**: `MOE`, `Segmentation`, `Nail`, `InputLine`, `Trie`
> **Related**: ../engine/composing.md, ../engine/continuous-candidate-display.md

---

## Summary

- MOE uses C++ closed-source engine + SWIG JNI interface
- Core concepts: InputLine (input line) + Nail (pinning) + Segmentation
- Data storage: Custom binary trie (tailo.tab), not SQLite
- Only supports Tailo (TL), not POJ

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                   Android App                        │
├─────────────────────────────────────────────────────┤
│  TaigiIME.java → ViewModel → Tailo.java (SWIG)      │
├─────────────────────────────────────────────────────┤
│                  TailoJNI.java                       │
├─────────────────────────────────────────────────────┤
│              C++ Native Library                      │
│         (Closed source, Trie + Segmentation)        │
├─────────────────────────────────────────────────────┤
│                  tailo.tab                           │
│            (Binary Trie Database)                    │
└─────────────────────────────────────────────────────┘
```

---

## Core Data Structures

### InputLine

Represents the line of text being input, contains state and cursor position.

```java
// Create and destroy
CreateInputLine(database) → InputLine
DestroyInputLine(inputLine)
ClearInputLine(inputLine)

// Input operations
InsertKey(inputLine, char)
RemoveKey(inputLine, Direction)  // FORWARD | BACKWARD

// Cursor control
GetCaretPosition(inputLine) → long
MoveBack(inputLine) → boolean
MoveForward(inputLine) → boolean
MoveHome(inputLine)
MoveEnd(inputLine)
MoveTo(inputLine, position) → boolean

// State query
GetInputLineSize(inputLine) → long
```

### Nail Mechanism

"Nail" is MOE's core concept, used for segment-by-segment candidate confirmation.

```java
// Get candidates at current nail position
GetNailPosition(inputLine) → long
GetCandidateCountAtNailPos(inputLine, vocType) → long
GetCandidatesAtNailPos(inputLine, vocType, offset, limit, handler)

// Confirm candidate (nail)
NailCandidate(inputLine, vocType, candidateIndex) → boolean
DoneNailing(inputLine) → boolean

// Commit confirmed text
PopFront(inputLine) → int
GetConfirmedCandidates(inputLine, handler)
```

**Flow Diagram:**

```
Input: "guasiaitaigi"
      ↓ Auto segment
Segments: [gua][si][ai][tai][gi]
          ↑
          NailPosition = 0

Select "我" → NailCandidate(0)
Segments: [我][si][ai][tai][gi]
              ↑
              NailPosition = 1

Select "是" → NailCandidate(1)
...continue similarly
```

### Segmentation

```java
// Get segmentation results
GetSegmentations(inputLine, handler)

// Toggle segmentation display mode
ToggleSegmentation(inputLine)

// Get segment info
GetKeySections(inputLine, pointer, vocType, handler)
```

**KeySectionsModel:**

```java
class KeySectionsModel {
    String composedCharacters;   // Confirmed part (e.g., "我是")
    String composingCharacters;  // Currently composing (e.g., "aitaigi")
}
```

---

## Candidate Model

### CandidateModel

```java
class CandidateModel {
    String mode;           // "hant" | "tailo"
    long position;         // Corresponding input position
    long sourceID;         // Source ID
    int spanUnits;         // Span units (important!)
    List<String> tailos;   // Multiple romanization readings
    boolean virtual;       // Whether dynamically generated
    String vocabulary;     // Vocabulary content
    double weight;         // Weight score
    int words;             // Word count
}
```

**spanUnits Explanation:**

```
Input: "taigi"
Candidate: "台語" (spanUnits=2) → Corresponds to [tai][gi] two segments
Candidate: "台" (spanUnits=1)   → Only corresponds to [tai] one segment
```

### VocType (Vocabulary Type)

```java
VT_HANT    // Hanzi
VT_TAILO   // Romanization
VT_MIXED   // Mixed (Han-lo parallel)
```

---

## Composition Mode

### CompositionMode

```java
CM_STANDARD  // Standard mode (Hanzi priority)
CM_EAZY      // Easy mode
CM_TAILO     // Pure romanization mode
```

---

## Learning Mechanism

### User Vocabulary vs Learned Vocabulary

```java
// User manually added words
AddUserVoc(database, key, value, score) → UserVocError
UpdateUserVoc(database, key, value, oldKey, oldValue)
EraseUserVoc(database, key, value)
GetUserVocs(database, prefix, ordering, desc, offset, limit, handler)
ClearUserVocs(database)

// System auto-learned
EraseLearnedVoc(database, key, value)
```

### LearnThreshold

```java
MAX_LEARNED_RULES          // Max learned rules
MAX_LEARNED_VOC_CANDIS     // Max learned vocabulary candidates
MAX_LEARNED_WORD_CANDIS    // Max learned word candidates
RIPE_RULE_APPROVALS        // Rule maturity approval count
RIPE_VOC_CANDI_APPROVALS   // Vocabulary candidate maturity approvals
RIPE_WORD_CANDI_APPROVALS  // Word candidate maturity approvals
```

---

## Related Choices

```java
// Get related choices from InputLine
GetCountOfRelatedChoices(inputLine, vocType) → long
GetRelatedChoices(inputLine, vocType, offset, limit, handler)
InsertRelatedChoice(inputLine, vocType, index) → boolean

// Get related choices from specific vocabulary
CreateRelatedChoicesFromVoc(database, key, value) → RelatedCache
GetCountOfRelatedChoicesFromVoc(cache, vocType) → long
GetRelatedChoicesFromVoc(cache, vocType, offset, limit, handler)
DestroyRelatedChoicesFromVoc(cache)
```

---

## Other Features

### Input Options

```java
EnableAutoAddSpuriousSpaces(inputLine, enabled)  // Auto add spaces
EnableFullShapeSymbols(inputLine, enabled)       // Full-width symbols
EnableIslandDoctrine(inputLine, enabled)         // Island doctrine (word boundaries)
EnableRawKeyOnly(inputLine, enabled)             // Raw input only
SetInputLineLimit(inputLine, limit)              // Input length limit
```

### Hanzi to Romanization

```java
Voc2tailos(database, hanzi, handler)  // "我" → ["gua2", "ngoo2"]
```

---

## Related Files

| File | Description |
|------|-------------|
| `references/moe_taigi_apk/decompiled/sources/moe/taigi/Tailo.java` | SWIG Wrapper |
| `references/moe_taigi_apk/decompiled/sources/moe/taigi/TailoJNI.java` | JNI method declarations |
| `references/moe_taigi_apk/decompiled/sources/android/moe/taiwanese/taigi/data/local/model/CandidateModel.java` | Candidate model |
| `references/moe_taigi_apk/decompiled/sources/android/moe/taiwanese/taigi/data/local/model/KeySectionsModel.java` | Segment model |
| `references/moe_taigi_apk/extracted/assets/tailo.tab` | Binary dictionary |

TaigiKeyboard's position on each MOE concept (Nail rejected; Han-lo mixed candidates, multiple readings and manual-vs-learned separation adopted) is recorded in [`mainstream-ime-comparison.md`](./mainstream-ime-comparison.md) card #13.
