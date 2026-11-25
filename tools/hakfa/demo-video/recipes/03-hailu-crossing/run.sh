#!/usr/bin/env bash
# Recipe 03: Continuous toneless HPP typing demo (Hailu, 30 keystrokes).
# Sentence: 渡細人仔過大路手愛牽等
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEMO_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

exec "$DEMO_DIR/generate.sh" \
  --platform both \
  --lect 2 \
  --input "Tusenginergotailushiuoikienden" \
  --name "03-hailu-crossing"
