#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
OUT_BASE="$SCRIPT_DIR/out"

PLATFORM="both"
INPUT="Iazagqiutienn'gamvoixiongoihiKiangjiangced"
LECT="1"
NAME="kipan-demo"
NOTE=""
LEARN_ARGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --platform) PLATFORM="$2"; shift 2 ;;
    --input)    INPUT="$2"; shift 2 ;;
    --lect)     LECT="$2"; shift 2 ;;
    --name)     NAME="$2"; shift 2 ;;
    --note)     NOTE="$2"; shift 2 ;;
    --learn)    LEARN_ARGS+=(--learn "$2"); shift 2 ;;
    -h|--help)
      echo "Usage: $0 [--platform mac|android|both] [--input <hpp>] [--lect 1..6] [--learn 'Hanji=rom'] [--note '<text>'] [--name <recipe-name>]"
      exit 0
      ;;
    *) echo "Unknown arg: $1" >&2; exit 1 ;;
  esac
done

OUT_DIR="$OUT_BASE/$NAME"

case "$LECT" in
  1) LECT_LABEL="四縣" ;;
  2) LECT_LABEL="海陸" ;;
  3) LECT_LABEL="大埔" ;;
  4) LECT_LABEL="饒平" ;;
  5) LECT_LABEL="詔安" ;;
  6) LECT_LABEL="南四縣" ;;
  *) LECT_LABEL="客話" ;;
esac

mkdir -p "$OUT_DIR" /tmp/swift-cache
TRACE_JSON="$OUT_DIR/${NAME}-trace.json"

echo "[1/3] Tracing real engine candidates..."
cp -f "$REPO_ROOT/engine/Cargo.lock" "$SCRIPT_DIR/trace/Cargo.lock"
CARGO_TARGET_DIR="$REPO_ROOT/engine/target" \
  cargo run --offline --manifest-path "$SCRIPT_DIR/trace/Cargo.toml" -- \
  --input "$INPUT" --lect "$LECT" --out "$TRACE_JSON" ${LEARN_ARGS[@]+"${LEARN_ARGS[@]}"}

encode_platform() {
  local plat="$1"
  local swift_script="$2"
  local gif_width="$3"
  local frames_dir="$OUT_DIR/frames_${plat}"

  echo "[2/3] Rendering ${plat} frames..."
  swift -module-cache-path /tmp/swift-cache "$swift_script" "$REPO_ROOT" "$TRACE_JSON" "$frames_dir" "$LECT_LABEL" "$NOTE"

  echo "[3/3] Encoding ${plat} MP4 & GIF..."
  ffmpeg -y -loglevel error -f concat -safe 0 -i "$frames_dir/concat.txt" \
    -fps_mode cfr -r 25 -pix_fmt yuv420p -c:v libx264 -crf 18 \
    "$OUT_DIR/${NAME}-${plat}.mp4"

  ffmpeg -y -loglevel error -f concat -safe 0 -i "$frames_dir/concat.txt" \
    -vf "fps=12,scale=${gif_width}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse" \
    -loop 0 "$OUT_DIR/${NAME}-${plat}.gif"

  ls -lh "$OUT_DIR/${NAME}-${plat}.mp4" "$OUT_DIR/${NAME}-${plat}.gif"
}

if [[ "$PLATFORM" == "mac" || "$PLATFORM" == "both" ]]; then
  encode_platform "mac" "$SCRIPT_DIR/render_mac.swift" 860
fi

if [[ "$PLATFORM" == "android" || "$PLATFORM" == "both" ]]; then
  encode_platform "android" "$SCRIPT_DIR/render_android.swift" 540
fi

OBS_TEMP="/Users/gjrobert/NAS/Obsidian/Main/!暫存"
if [[ -d "$OBS_TEMP" ]]; then
  OBS_OUT="$OBS_TEMP/demo-video/$NAME"
  mkdir -p "$OBS_OUT"
  cp -f "$OUT_DIR"/*.mp4 "$OUT_DIR"/*.gif "$OBS_OUT"/ 2>/dev/null || true
  echo "Synced .mp4 / .gif to $OBS_OUT"
fi
