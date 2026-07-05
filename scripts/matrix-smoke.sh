#!/usr/bin/env bash
set -euo pipefail

: "${OPENAI_API_KEY:?set OPENAI_API_KEY, e.g. OPENAI_API_KEY=\"$(omp auth-gateway token)\"}"
export OPENAI_BASE_URL=${OPENAI_BASE_URL:-http://127.0.0.1:4000/v1}
export OPENAI_MODEL=${OPENAI_MODEL:-google-antigravity/gemini-3.5-flash}

TEXT_PATH=${TEXT_PATH:-fixtures/demo.txt}
MATRIX_PATH=${MATRIX_PATH:-matrices/smoke.json}
PROBES_PATH=${PROBES_PATH:-probes/demo.json}
RUN_DIR=${RUN_DIR:-runs/smoke}

cargo run -- matrix \
  --text "$TEXT_PATH" \
  --matrix "$MATRIX_PATH" \
  --probes "$PROBES_PATH" \
  --out "$RUN_DIR"
