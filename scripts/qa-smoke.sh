#!/usr/bin/env bash
set -euo pipefail

: "${OPENAI_API_KEY:?set OPENAI_API_KEY}"
: "${OPENAI_BASE_URL:?set OPENAI_BASE_URL}"
: "${OPENAI_MODEL:?set OPENAI_MODEL}"

TEXT_PATH=${TEXT_PATH:-fixtures/corpus/visual-context-research/source.txt}
IMAGE_PATH=${IMAGE_PATH:-runs/qa-smoke/context.png}
QA_PATH=${QA_PATH:-fixtures/corpus/visual-context-research/qa.json}
RUN_DIR=${RUN_DIR:-runs/qa-smoke}
FONT_PATH=${FONT_PATH:-fonts/zpix.ttf}
FONT_SIZE=${FONT_SIZE:-9}
THRESHOLD=${THRESHOLD:-0.30}
LINE_SPACING=${LINE_SPACING:-0}
MAX_WIDTH=${MAX_WIDTH:-750}
MAX_TOKENS=${MAX_TOKENS:-160}

mkdir -p "$RUN_DIR"

cargo run -- render \
  --text "$TEXT_PATH" \
  --out "$IMAGE_PATH" \
  --font "$FONT_PATH" \
  --font-size "$FONT_SIZE" \
  --threshold "$THRESHOLD" \
  --line-spacing "$LINE_SPACING" \
  --max-width "$MAX_WIDTH"

cargo run -- qa \
  --image "$IMAGE_PATH" \
  --qa "$QA_PATH" \
  --out "$RUN_DIR/qa.jsonl" \
  --max-tokens "$MAX_TOKENS"
