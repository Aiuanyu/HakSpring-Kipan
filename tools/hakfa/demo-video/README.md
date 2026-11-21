# Kipan Demo Video Generator (`tools/hakfa/demo-video`)

Generates Mac and Android typing demo videos (`.mp4` and `.gif`) driven by the real Rust engine (`assets/dictionaries/` + `dispatch`).

## Prerequisites (macOS)

- Rust toolchain (`cargo`) + `protoc` in `PATH`
- Swift (included with Xcode)
- `ffmpeg` (`brew install ffmpeg`)
- Optional font: `Carlito` installed in user/system fonts (falls back to system sans-serif); `Maruko Gothic CJKtc` is loaded automatically from `assets/fonts/font/marukogothic_regular.ttf`.

## Directory Layout

- `recipes/<name>/run.sh`: Per-video lightweight recipe scripts (committed to Git).
- `out/<name>/`: Per-video generated artifacts (`*.mp4`, `*.gif`, `frames_*`, `*-trace.json` — gitignored).

## Usage

Run an existing recipe:

```bash
./tools/hakfa/demo-video/recipes/01-continuous-typing/run.sh
```

Or invoke `generate.sh` directly:

```bash
./tools/hakfa/demo-video/generate.sh \
  --platform both \
  --lect 1 \
  --input "Iazagqiutienn'gamvoixiongoihiKiangjiangced" \
  --learn "輕井澤=kiáng-jiàng-ced" \
  --note "（「輕井澤」非內建詞，可另參考自動學習建詞功能！）" \
  --name "01-continuous-typing"
```
