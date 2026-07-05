#!/usr/bin/env bash
set -euo pipefail

TEXT_PATH=${TEXT_PATH:-fixtures/demo.txt}
IMAGE_PATH=${IMAGE_PATH:-output_long.png}
FONT_PATH=${FONT_PATH:-fonts/zpix.ttf}
FONT_SIZE=${FONT_SIZE:-12}
THRESHOLD=${THRESHOLD:-0.30}
LINE_SPACING=${LINE_SPACING:-6}
MAX_WIDTH=${MAX_WIDTH:-750}

cargo run -- render \
  --text "$TEXT_PATH" \
  --out "$IMAGE_PATH" \
  --font "$FONT_PATH" \
  --font-size "$FONT_SIZE" \
  --threshold "$THRESHOLD" \
  --line-spacing "$LINE_SPACING" \
  --max-width "$MAX_WIDTH"
