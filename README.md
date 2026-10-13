# TaigiKeyboard 台語齒盤

A Taiwanese input method for iOS, Android, macOS, Windows, and Linux.

![iOS 17+](https://img.shields.io/badge/iOS-17%2B-blue)
![Android 9+](https://img.shields.io/badge/Android-9%2B-green)
![macOS 14+](https://img.shields.io/badge/macOS-14%2B-lightgrey)
![Windows 10+](https://img.shields.io/badge/Windows-10%2B-blue)
![Linux Fcitx5 | IBus](https://img.shields.io/badge/Linux-Fcitx5%20%7C%20IBus-orange)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue)](LICENSE)

## Download

| Platform | Where |
| --- | --- |
| iOS / iPadOS | [App Store](https://apps.apple.com/app/id6751871806) |
| Android | [Google Play](https://play.google.com/store/apps/details?id=com.siansiansu.taigikeyboard) |
| macOS / Windows | [taigikeyboard.tw](https://taigikeyboard.tw) |
| Linux (`.deb`, `.rpm`, Arch; Fcitx5 or IBus) | [GitHub Releases](https://github.com/taigikeyboard/taigikeyboard/releases) |

Installation and setup for every platform: [`docs/INSTALL.md`](docs/INSTALL.md).

## Features

- Romanization input in POJ, TL, and TPS
- Hanji and mixed Hanji–Romanization output
- Continuous multi-syllable typing
- Tone marks, tone numbers, and tone variation
- Autocomplete and next-word prediction
- Custom dictionary and frequency learning
- Lossless TL ↔ POJ ↔ TPS conversion

## How it works

One Rust engine (`engine/`) holds phonetics, composing, lexicon, ranking and user data. Each platform is a thin shell: it owns the text field, the candidate window and settings, and every request reaches the engine's `dispatch` as a protobuf envelope.

```text
┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐
│ iOS          │ │ Android      │ │ macOS        │ │ Windows      │ │ Linux        │
│ Swift        │ │ Kotlin       │ │ Swift        │ │ Rust         │ │ Fcitx5 (C++) │
│ KeyboardKit  │ │ Compose      │ │ IMKit        │ │ TSF          │ │ IBus (Rust)  │
└──────┬───────┘ └──────┬───────┘ └───┬─────┬────┘ └──────┬───────┘ └──────┬───────┘
       │                │             │     │             │                │ C ABI
       │                │             │     │ macos-ffi   │ path dep       │ crate
       │                │             │   ┌─┴─────────────┴────────────────┴───────┐
       │                │             │   │ desktop/  taigi-desktop-core           │
       │                │             │   │ keys · intent executor · engine bridge │
       │                │             │   └───────────────────┬────────────────────┘
       │                │             │                       │
       │ swift-ffi      │ JNI         │ swift-ffi             │ path dep
       │                │             │ (user data)           │
       │                │             │                       │
┌──────┴────────────────┴─────────────┴───────────────────────┴────────────────────┐
│ engine/  dispatch::process_request — proto envelope in, proto out                │
│                                                                                  │
│   composing · lexicon · ranking · nextword · userdata                            │
│   phonetics (POJ / TL / TPS) · protos · mmap-host                                │
└──────────────┬──────────────────────────────────────────────┬────────────────────┘
               │ mmap, read-only                              │ read / write
               │                                              │
┌──────────────┴───────────────────────┐    ┌─────────────────┴────────────────────┐
│ assets/dictionaries/  (committed)    │    │ engine/userdata  (SQLite, on device) │
│ dictionary.fst · dictionary.bin      │    │ user_frequency · user_association    │
│ association.bin · syllables.fst      │    │ custom_dictionary · learned_phrases  │
└──────────────▲───────────────────────┘    └──────────────────────────────────────┘
               │ make dict
               │
┌──────────────┴───────────────────────┐
│ dictionary/  sources → normalize     │
│ (taigi-converter) → merge → build    │
└──────────────────────────────────────┘
```

- **iOS / Android** call the engine library directly (`swift-ffi` static library, `android-jni` shared library).
- **macOS** runs its key path through `taigi-macos-ffi` into the shared `desktop/` core; its user-data pages call `swift-ffi` in the same library.
- **Windows / Linux** link the engine crates in-process. On Linux the Fcitx5 addon (C++) calls the Rust core over a C ABI, and IBus talks over D-Bus to a Rust engine process; both run `taigi-linux-core`.

Crate graph, build pipeline and keystroke data flow: [`docs/architecture/system-overview.md`](docs/architecture/system-overview.md).

## Building

See [`docs/BUILDING.md`](docs/BUILDING.md). Clone with `--recurse-submodules`, then run `make build` once.

## Contributing

- [`CONTRIBUTING.md`](CONTRIBUTING.md) — build, test, and submit changes
- [`docs/README.md`](docs/README.md) — architecture and engine docs
- [`SECURITY.md`](SECURITY.md) — vulnerability reporting and data handling
- [`CHANGELOG.md`](CHANGELOG.md) — release history

## License

Source code: [Apache License 2.0](LICENSE).

**Dictionary data is not Apache-licensed.** Each source keeps its own terms, and one carries a NonCommercial clause, so the compiled dictionary shipped in every app is non-commercial. See [`dictionary/LICENSE`](dictionary/LICENSE) for per-source terms and required attribution, and [`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md) for fonts and vendored code. Code signing: [`docs/CODE_SIGNING_POLICY.md`](docs/CODE_SIGNING_POLICY.md).
