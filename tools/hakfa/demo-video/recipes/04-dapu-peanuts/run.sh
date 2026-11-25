#!/usr/bin/env bash
# Recipe 04: Continuous toneless HPP typing demo (Dapu, 34 keystrokes).
# Sentence: 阿爹肚屎疾阿奶講使毋得食地豆
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEMO_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

exec "$DEMO_DIR/generate.sh" \
  --platform both \
  --lect 3 \
  --input "Adadushicidanegongsiimdedshidtiteu" \
  --name "04-dapu-peanuts"
