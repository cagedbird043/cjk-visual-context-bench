#!/usr/bin/env bash
set -euo pipefail

: "${OPENAI_API_KEY:?set OPENAI_API_KEY}"
: "${OPENAI_BASE_URL:?set OPENAI_BASE_URL}"
: "${OPENAI_MODEL:?set OPENAI_MODEL}"

IMAGE_PATH=${IMAGE_PATH:-output_long.png}
SOURCE_PATH=${SOURCE_PATH:-fixtures/corpus/demo-tech-report/source.txt}
PROMPT_PATH=${PROMPT_PATH:-prompts/ocr.txt}
RUN_DIR=${RUN_DIR:-runs/ocr-smoke}
MAX_TOKENS=${MAX_TOKENS:-4096}

cargo run -- ocr \
  --image "$IMAGE_PATH" \
  --source "$SOURCE_PATH" \
  --prompt "$PROMPT_PATH" \
  --out "$RUN_DIR/transcript.txt" \
  --max-tokens "$MAX_TOKENS"
