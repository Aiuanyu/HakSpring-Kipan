#!/usr/bin/env bash
# Recipe 02: Continuous toneless HPP typing demo B (Sixian, 53 keystrokes).
# Note: uses a temporary learned phrase for "問𠊎" so 問 wins over 門 in the first clause.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEMO_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

exec "$DEMO_DIR/generate.sh" \
  --platform both \
  --lect 1 \
  --input "Apomunngaizonvugkasiidzumokisiidngaihancaitoibedzobogo" \
  --learn "問𠊎=mun-ngǎi" \
  --note "（前段「問𠊎」先以自學詞條示範；實際首次選字後即會自動記住！）" \
  --name "02-grandma-lunch"
