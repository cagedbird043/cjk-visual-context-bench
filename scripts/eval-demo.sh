#!/usr/bin/env bash
set -euo pipefail

: "${OPENAI_API_KEY:?set OPENAI_API_KEY, e.g. OPENAI_API_KEY=\"$(omp auth-gateway token)\"}"
export OPENAI_BASE_URL=${OPENAI_BASE_URL:-http://127.0.0.1:4000/v1}
export OPENAI_MODEL=${OPENAI_MODEL:-google-antigravity/gemini-3.5-flash}

IMAGE_PATH=${IMAGE_PATH:-output_long.png}
PROBES_PATH=${PROBES_PATH:-probes/demo.json}

cargo run -- eval \
  --image "$IMAGE_PATH" \
  --probes "$PROBES_PATH"
