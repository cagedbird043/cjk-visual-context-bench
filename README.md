# CJK Visual Context Bench

Experimental benchmark for dense bitmap visual-context compression.

Goal: turn real CJK-heavy developer context into compact bitmap images, ask vision models to read them, and measure whether the context remains usable.

This is not a paper repo. It is an experiment repo: render, ask, score, publish data.

## Current research question

Can a bitmap image carry real developer context reliably enough for LLM context compaction?

The hard part is not Chinese prose only. Real developer context includes:

- Chinese and English text
- model IDs
- file paths
- URLs
- commands
- versions
- hashes
- JSON keys
- error codes
- candidate IDs

The benchmark therefore measures both:

- semantic QA: does the model understand the context?
- exact QA: can the model recover exact developer values, character-for-character?

## Current finding

Compact stream rendering is mandatory. Bitmap images must not preserve document layout.

No headings, blank lines, indentation, paragraph spacing, or page-like layout should survive in the image. Layout wastes pixels. Structure belongs in corpus metadata and questions, not in the bitmap.

Early results:

- ordinary anti-aliased CJK fonts fail under dense 1-bit rendering
- pixel fonts are the right search space
- zpix is the current baseline
- exact values expose glyph ambiguity such as `l0` vs `10` and `w750` vs `v750`
- larger font sizes may be better engineering points after layout removal

## Setup

```sh
cargo check
scripts/fetch-fonts.sh
```

Fonts are downloaded on demand. Font binaries are not committed. Exact sources, versions, hashes, and licenses live in:

```text
fonts/fonts.lock.json
```

## Run examples

Set any OpenAI-compatible vision endpoint:

```sh
export OPENAI_BASE_URL=http://127.0.0.1:4000/v1
export OPENAI_API_KEY=...
export OPENAI_MODEL=google-antigravity/gemini-3.5-flash
```

Render a compact bitmap:

```sh
cargo run -- render \
  --text fixtures/corpus/exact-dev-context/source.txt \
  --out runs/example/context.png \
  --font fonts/zpix.ttf \
  --font-size 12 \
  --threshold 0.30 \
  --line-spacing 0 \
  --max-width 750
```

Run exact-value QA:

```sh
cargo run -- qa \
  --image runs/example/context.png \
  --qa fixtures/corpus/exact-dev-context/exact_qa.json \
  --out runs/example/exact.jsonl
```

Run semantic QA:

```sh
cargo run -- qa \
  --image runs/example/context.png \
  --qa fixtures/corpus/exact-dev-context/semantic_qa.json \
  --out runs/example/semantic.jsonl
```

## Repository layout

```text
src/                 Rust renderer, model calls, scorers
fixtures/corpus/     benchmark corpora
matrices/            render parameter grids
scripts/             reproducible shell entrypoints
fonts/fonts.lock.json font asset lockfile
runs/                generated results, gitignored
```

## Scoring

Exact QA:

```text
score = exact
wrong one character = fail
```

Semantic QA:

```text
score = semantic
normalized char-F1 after whitespace/punctuation normalization
```

OCR exists only as a diagnostic. Product usefulness is judged by QA, exact value recovery, hallucination/refusal behavior, and pixel cost.

## Status

Research prototype. Public data and reports will grow from batch benchmark runs.
