#!/usr/bin/env bash
# Recipe 01: Continuous toneless HPP typing demo (Sixian, 42 keystrokes).
# Note: uses a temporary learned phrase for "輕井澤" (not in bundled lexicon).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEMO_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

exec "$DEMO_DIR/generate.sh" \
  --platform both \
  --lect 1 \
  --input "Iazagqiutienn'gamvoixiongoihiKiangjiangced" \
  --learn "輕井澤=kiáng-jiàng-ced" \
  --note "（「輕井澤」非內建詞，可另參考自動學習建詞功能！）" \
  --name "01-continuous-typing"
