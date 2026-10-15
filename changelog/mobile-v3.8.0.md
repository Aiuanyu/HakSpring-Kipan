## v3.8.0

Mobile release for iOS and Android. Headline: **Custom Layouts** — build up to
five of your own romanization keyboards from a built-in layout: swap keys, put
in spare letters, finals or symbols, remove keys. Both phones also gain
action-field Enter, double-space period in Taigi and English, a rewritten typing
guide, and the Kautian contracted readings (合音唸作). Android adds spacebar
cursor swipe; iOS adds the `@` / `/` key in email and web fields and a Return
icon that follows the field. Desktop work in the range (two chords per
composing action) belongs to desktop v3.8.0 and does not touch phones.

### Shared (iOS + Android)

#### New

- **Custom Layouts.** Layout → Custom Layouts → New Layout… opens an editor:
  pick a base (PhahTaigi, Lohankha, MOE Layout 1, MOE Layout 2), then edit the
  drawn keyboard. (#505, #506, #507)
  - Tap two keys, or drag one onto another, to swap them. Function keys swap
    within their own row.
  - Tap a key then a spare, or drag a spare onto a key, to replace it. The
    spare tray holds a–z, `o͘`, `-`, the multi-letter keys (`ph ts tsh oo th
    kh ng nn ch chh`), 16 finals (`an ia in au ong ian un ai iu ann ang ui inn
    iau er ir`), `ee`, and the symbol page. (#508, #510)
  - Drag a key to the tray to remove it; an empty row leaves the keyboard. The
    emoji key can be removed too.
  - Save names the layout and applies it. Up to five; each card can be applied,
    edited or deleted. The in-keyboard layout picker lists them. Phonetic
    Symbols mode sets a custom layout aside and brings it back on leaving.
- **Double-space period.** Two quick spaces after a letter or number type
  `. `, after full-width punctuation `。`, in Taigi and English, outside
  composition. (#499)

#### Improved

- **Enter in Send / Search / Go fields** while composing only confirms the
  typed word, with no auto space; the next Enter runs the action. (#499)
- **Picking an English suggestion** follows Auto-Space. (#499)

#### Changed

- **Typing guide rewritten.** 13 entries with new light and dark pictures;
  new entries for one-handed mode, display modes, Learning Records and typing
  settings. The info buttons beside Annotate in Brackets and Auto-Space show
  the typing-settings summary. (#497)
- Wording: Auto-Space reads 自動插入空白 in Taigi; 漢羅合用 is read
  `ha̍p-iōng`. (#497)
- **Next-word suggestion with Annotate in Brackets and no reading** writes
  `台語`, not `台語 ()`. (#503, #504)

#### Bug Fixes

- Settings parity between the phones: Learning Records shows Count / Last Used
  labels on both; import / export errors use the same generic messages; the
  copyright page lists the STTI dictionary on both. (#498)
- The Japanese name of the GenYoGothic font is 源樣烏體. (#496)

### iOS

#### Improved

- **Email and web-address fields get an `@` / `/` key** in place of the
  bottom-row `,` (or after the emoji key). (#502)
- **The idle Return key icon follows the field** (done, go, search, send,
  new line). (#502)
- **English mode long-press shows symbols only**, no Taigi tone or MOE
  options. (#502)
- **Half-width ellipsis key** `…`, long-press `⋯ ‥`. (#502)
- The candidate list is no longer cut at 200 entries. (#502)
- One-shot Shift is used up by letters only. (#499)

#### Bug Fixes

- Reset to Defaults also turns the custom dictionary back on and clears the
  saved Phonetic Symbols layout. (#498)

### Android

#### New

- **Swipe across Space to move the cursor.** Long-press still opens the input
  method picker; disabled while composing. (#500)

#### Changed

- **The globe key sits right after the symbols key.** (#501)
- Three-letter keys shrink to fit on every layout; picker cards fit `chh`.
  (#506, #514)

#### Bug Fixes

- The setup guide has its own Android wording and no longer reopens when the
  screen rotates. (#498)
- Custom Dictionary shows a loading row instead of flashing the empty card.
  (#498)
- The keyboard button's long-press has the accessibility name One-handed mode.
  (#501)

### Dictionary

- Kautian 合音唸作 contracted readings are kept (49 rows): `lueh` gives 落去,
  `tsa̋u-kiánn` gives 查某囝. (#509)
