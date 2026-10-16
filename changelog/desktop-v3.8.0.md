# desktop v3.8.0

Two keys per action, and better candidates. The Output rows on a shorter
Shortcuts pane now take two keys, and Space joins Enter as a Confirm Key.
Candidates that cover everything you typed come before ones that cover only
the start, and your own Custom Dictionary words follow the words you pick and
no longer appear after only a letter or two. Phonetic Symbols keeps the tone
marks and spaces you type. The MOE Dictionary gains a separate Alternative
Readings switch and 49 contracted readings such as `lueh` 落去. Upgrade from
v3.7.0.

### macOS

#### Input

- **Space confirms, like Enter.** On an install that never changed its
  shortcuts, Space now writes what you are typing in the current script.
  Output the Other Script (Hanji / Romanization) has no key by default; record
  one on the Shortcuts pane if you want it back. ⇧ with a candidate's number
  still writes that candidate in the other script. With Phonetic Symbols, Space
  in the middle of a word with no candidate list open still separates
  syllables. A key you recorded yourself is kept. (#512)
- **Phonetic Symbols keeps the tone marks and spaces you type.** `ㄉㄞ˫ㄨㄢ`
  gives 台灣 with the tone you typed. A tone mark or a space ends a syllable,
  so `ㄍㄚ␣ㄍㄚ` no longer joins into one word across the space. (#479)
- **Phonetic Symbols lists the shorter first syllable.** For `ㄌㄧㄏㄛ` the
  candidate list now offers 你 and 汝, matching the 你好 conversion, instead of
  only 裂 / 哩. The same applies to `ㄍㄚㄉㄚ`. (#477)

#### Candidates

- **Whole-input candidates come first.** A candidate covering everything you
  typed now comes before ones covering only the first syllables: `taiuanl`
  gives 台灣人 first, `taig` 台語. (#482)
- **Custom Dictionary words follow the dictionary's rules.** Words you have
  picked come first; a custom word you never picked comes next, ahead of the
  rest of the dictionary. A custom word reached by its abbreviation comes after
  the full readings: `thng` lists 湯 and 糖 before a custom 退伍. A custom word
  is no longer offered after just one or two letters: a custom 陳建中 appears
  from `tankiant` or its abbreviation `tkt`, not from `t` or `ta`. (#481, #482,
  #515, #517)
- **49 contracted readings from the MOE Dictionary** (合音唸作), such as
  `lueh` 落去 and `tsa̋u-kiánn` 查某囝. (#509)
- About 50 MOE Dictionary words with 一 now have a space where they had a
  hyphen (`tsi̍t kang`), following the dictionary's latest data.

#### Settings

- **Two keys per Output action on the Shortcuts pane.** Confirm Key, Output
  What You Are Typing and Output the Other Script each have two fields, and
  either key does the action. Clearing one field leaves the other. Confirm Key
  ships with Enter and Space. (#512, #521)
- **A shorter Shortcuts pane.** The rows that only listed fixed keys are gone:
  Delete What You Are Typing (Esc still cancels), Move Through Candidates (the
  arrow and page keys still move) and Pick a Candidate Directly (the candidate
  window shows each key). The ⇧ number keys now sit under Output the Other
  Script. The global shortcuts are split into Romanization, Phonetic Symbols
  and Other. (#518, #520, #523, #524)
- **Delete All Custom Words and Delete Learning Records run on press**, with no
  confirmation dialog. A notice appears only when it fails. Export a `.taigi`
  backup first if you may want the data back. (#519)
- **Hanji labels write 的 for ê**, for example 修正拍的字 and 輸出拍的字.
  Romanized labels keep ê. (#522)
- **Alternative Readings (又唸作) has its own switch** under the MOE Dictionary,
  on by default. Turn it off to drop the readings the dictionary lists only as
  alternatives. (#474)

#### Updates

- The in-app update downloads the installer from `dl.taigikeyboard.tw`, a
  mirror much faster from Taiwan than GitHub.

### Windows

#### Input

- **Space confirms, like Enter.** On an install that never changed its
  shortcuts, Space now writes what you are typing in the current script.
  Output the Other Script (Hanji / Romanization) has no key by default; record
  one on the Shortcuts pane if you want it back. Shift with a candidate's number
  still writes that candidate in the other script. With Phonetic Symbols, Space
  in the middle of a word with no candidate list open still separates
  syllables. A key you recorded yourself is kept. (#512)
- **Phonetic Symbols keeps the tone marks and spaces you type.** `ㄉㄞ˫ㄨㄢ`
  gives 台灣 with the tone you typed. A tone mark or a space ends a syllable,
  so `ㄍㄚ␣ㄍㄚ` no longer joins into one word across the space. (#479)
- **Phonetic Symbols lists the shorter first syllable.** For `ㄌㄧㄏㄛ` the
  candidate list now offers 你 and 汝, matching the 你好 conversion, instead of
  only 裂 / 哩. The same applies to `ㄍㄚㄉㄚ`. (#477)

#### Candidates

- **Whole-input candidates come first.** A candidate covering everything you
  typed now comes before ones covering only the first syllables: `taiuanl`
  gives 台灣人 first, `taig` 台語. (#482)
- **Custom Dictionary words follow the dictionary's rules.** Words you have
  picked come first; a custom word you never picked comes next, ahead of the
  rest of the dictionary. A custom word reached by its abbreviation comes after
  the full readings: `thng` lists 湯 and 糖 before a custom 退伍. A custom word
  is no longer offered after just one or two letters: a custom 陳建中 appears
  from `tankiant` or its abbreviation `tkt`, not from `t` or `ta`. (#481, #482,
  #515, #517)
- **49 contracted readings from the MOE Dictionary** (合音唸作), such as
  `lueh` 落去 and `tsa̋u-kiánn` 查某囝. (#509)
- About 50 MOE Dictionary words with 一 now have a space where they had a
  hyphen (`tsi̍t kang`), following the dictionary's latest data.

#### Settings

- **Two keys per Output action on the Shortcuts pane.** Confirm Key, Output
  What You Are Typing and Output the Other Script each have two fields, and
  either key does the action. Clearing one field leaves the other. Confirm Key
  ships with Enter and Space. (#512, #521)
- **A shorter Shortcuts pane.** The rows that only listed fixed keys are gone:
  Delete What You Are Typing (Esc still cancels), Move Through Candidates (the
  arrow and page keys still move) and Pick a Candidate Directly (the candidate
  window shows each key). The Shift number keys now sit under Output the Other
  Script. The global shortcuts are split into Romanization, Phonetic Symbols
  and Other. (#518, #520, #523, #524)
- **Delete All Custom Words and Delete Learning Records run on press**, with no
  confirmation dialog. A notice appears only when it fails. Export a `.taigi`
  backup first if you may want the data back. (#519)
- **Hanji labels write 的 for ê**, for example 修正拍的字 and 輸出拍的字.
  Romanized labels keep ê. (#522)
- **Alternative Readings (又唸作) has its own switch** under the MOE Dictionary,
  on by default. Turn it off to drop the readings the dictionary lists only as
  alternatives. (#474)

#### Updates

- The in-app update downloads the installer from `dl.taigikeyboard.tw`, a
  mirror much faster from Taiwan than GitHub.

### Linux

#### Input

- **Space confirms, like Enter.** On an install that never changed its
  shortcuts, Space now writes what you are typing in the current script.
  Output the Other Script (Hanji / Romanization) has no key by default; record
  one on the Shortcuts pane if you want it back. Shift with a candidate's number
  still writes that candidate in the other script. With Phonetic Symbols, Space
  in the middle of a word with no candidate list open still separates
  syllables. A key you recorded yourself is kept. (#512)
- **Phonetic Symbols keeps the tone marks and spaces you type.** `ㄉㄞ˫ㄨㄢ`
  gives 台灣 with the tone you typed. A tone mark or a space ends a syllable,
  so `ㄍㄚ␣ㄍㄚ` no longer joins into one word across the space. (#479)
- **Phonetic Symbols lists the shorter first syllable.** For `ㄌㄧㄏㄛ` the
  candidate list now offers 你 and 汝, matching the 你好 conversion, instead of
  only 裂 / 哩. The same applies to `ㄍㄚㄉㄚ`. (#477)

#### Candidates

- **Whole-input candidates come first.** A candidate covering everything you
  typed now comes before ones covering only the first syllables: `taiuanl`
  gives 台灣人 first, `taig` 台語. (#482)
- **Custom Dictionary words follow the dictionary's rules.** Words you have
  picked come first; a custom word you never picked comes next, ahead of the
  rest of the dictionary. A custom word reached by its abbreviation comes after
  the full readings: `thng` lists 湯 and 糖 before a custom 退伍. A custom word
  is no longer offered after just one or two letters: a custom 陳建中 appears
  from `tankiant` or its abbreviation `tkt`, not from `t` or `ta`. (#481, #482,
  #515, #517)
- **49 contracted readings from the MOE Dictionary** (合音唸作), such as
  `lueh` 落去 and `tsa̋u-kiánn` 查某囝. (#509)
- About 50 MOE Dictionary words with 一 now have a space where they had a
  hyphen (`tsi̍t kang`), following the dictionary's latest data.

#### Settings

- **Two keys per Output action on the Shortcuts pane.** Confirm Key, Output
  What You Are Typing and Output the Other Script each have two fields, and
  either key does the action. Clearing one field leaves the other. Confirm Key
  ships with Enter and Space. (#512, #521)
- **A shorter Shortcuts pane.** The rows that only listed fixed keys are gone:
  Delete What You Are Typing (Esc still cancels), Move Through Candidates (the
  arrow and page keys still move) and Pick a Candidate Directly (the candidate
  window shows each key). The Shift number keys now sit under Output the Other
  Script. The global shortcuts are split into Romanization, Phonetic Symbols
  and Other. (#518, #520, #523, #524)
- **Delete All Custom Words and Delete Learning Records run on press**, with no
  confirmation dialog. A notice appears only when it fails. Export a `.taigi`
  backup first if you may want the data back. (#519)
- **Hanji labels write 的 for ê**, for example 修正拍的字 and 輸出拍的字.
  Romanized labels keep ê. (#522)
- **Alternative Readings (又唸作) has its own switch** under the MOE Dictionary,
  on by default. Turn it off to drop the readings the dictionary lists only as
  alternatives. (#474)
