## v3.7.0

Mobile release for iOS and Android. Headline: **conversion follows written
Taigi** — segmentation, the first candidate and the candidate list now rank by
how often a Taigi text corpus uses each word. Typed tone digits stay tone marks,
a candidate covering the whole input comes first, and custom words follow the
same reach rule as dictionary words. Phonetic Symbols gets three typing fixes.
iOS fixes a keyboard-controller leak; Android matches iOS key geometry and
keeps half-typed text when the keyboard hides. Much of the range is desktop
work (desktop v3.7.0) with no effect on phones.

### Shared (iOS + Android)

#### New

- **Alternative Readings toggle.** Manage Dictionaries → MOE dictionary gains
  Alternative Readings (又唸作), default on. Turning it off hides the 1,764
  readings that come only from the 又唸作 sheet. Default on matches the previous
  always-on behavior. (#474)

#### Improved

- **Conversion picks the word written Taigi uses most.** Segmentation and the
  first candidate come from corpus word counts instead of a mix of character
  and word counts. `kau3siu7` and `kausiu` give 教授 (were 到受 and 狗岫).
  (#461, #462, #463, #466)
- **The candidate list and dictionary search follow the same counts.** Words
  after the first candidate are ordered by the same score; Dictionary search
  results too, Kautian entries first. (#468)
- **Tone digits stay tone marks.** `tai5gi2` shows `tâigí`; `tai1` shows `tai`.
  Runs of digits stay literal (`covid19`, `mp3`). The preedit, the typed-text
  candidate and the committed text agree. (#449, #450)
- **A typed hyphen closes a syllable.** `siam-tioh` leads with 閃 instead of 寫,
  and a digit right after the hyphen is that syllable's tone (`tai-2` → 台).
  (#454)
- **A candidate covering the whole input comes first, and custom words lead
  their tier.** `taiuanl` → 台灣人 is first (was 76th). (#482)
- **Custom words follow the syllable-reach rule.** A custom 陳建中
  `Tân Kiàn-tiong` no longer shows from `t`, `ta` or `tan`; it appears from
  `tankiant` and is first at `tankiantiong`. (#481)

#### Bug Fixes

- **The same word is listed once.** `li2` no longer shows 李 twice; a custom
  `tai5-gi2` / 台語 merges with the dictionary's `tâi-gí` / 台語. (#447)
- **Phonetic Symbols:**
  - ㆳ (on the ㆪ long-press) finds words; it used to return none. (#443)
  - A shorter first syllable is listed when a longer one straddles it:
    `ㄌㄧㄏㄛ` also lists 你 / 汝. (#477)
  - Typed tone marks and spaces are kept: `ㄉㄞ˫ㄨㄢ` gives 台灣, and
    `ㄍㄚ␣ㄍㄚ` no longer gives 角仔. (#479)

#### Changed

- **Layout page grouped by script.** Shelves: Custom Layouts (coming soon),
  Universal, Tâi-lô / POJ, Phonetic Symbols. Picking a Tâi-lô / POJ card also
  switches the input mode; the in-keyboard layout picker shows the POJ picture
  in POJ mode. Theme and layout card pictures are redrawn from the real
  keyboard. (#469, #470, #471, #472)
- **Custom dictionary readings tidied once on upgrade.** Unicode is normalized
  (NFC) and tone digits become marks (`tsui2-ong5-tshut` →
  `tsuí-ông-tshut`). Hanji, counts and row count are kept. New entries and
  CSV / `.taigi` imports are stored the same way. (#448, #453)
- Wording: 教育部排列 → 教育部佈局; the reset row reads Reset to Defaults
  (恢復預設設定); Show Typed Text First and the POJ nasal-mark labels revised in
  Taigi. (#441)

### iOS

#### Bug Fixes

- **Keyboard memory stays steady.** Each re-show of the keyboard left the old
  keyboard controller alive; it is now released, keys leaving the screen ignore
  taps, and next-word suggestions stop for a dismissed keyboard. (#480)
- **Full-width symbols type `／`** instead of ASCII `/`. (#493)

### Android

#### Changed

- **Key size and spacing match iOS.** Keys are 38 dp tall (were 42 dp) with new
  gaps; shift and delete take 13 % of the row, enter 15 %. (#456)

#### Bug Fixes

- **Half-typed text survives hiding the keyboard.** Hiding and re-showing the
  keyboard in the same field no longer deletes the composition. (#494)
- **The candidate strip no longer slides under the expand arrow.** (#435)
- **Reset to Defaults leaves the globe key off**, as on a fresh install.
  (#493)

### Dictionary

- `dictionary.bin` format v4 carries corpus word counts and word-pair data
  (`walker_cost`). (#461, #462)
- Kautian refresh: about 50 一X measure words take a space instead of a hyphen
  (`tsi̍t-kang` → `tsi̍t kang`); 一日 gains `tsi̍t ji̍t` / `tsi̍t li̍t`. (#478)
- Readings found only in 又唸作 form their own source (1,764 rows). (#474)
