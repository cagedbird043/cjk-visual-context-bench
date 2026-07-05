# CJK Visual Context Bench

Benchmark for replacing OMP `snapcompact` image rendering with a CJK-capable bitmap archive renderer.

Goal: take full OMP conversation history, serialize it like `packages/snapcompact`, render dense bitmap frames under OMP-like frame budgets, then measure whether a vision model can continue from those frames with the same recall quality as current `snapcompact`.

This repo is experiment-first: real contexts, rendered frames, QA recall, exact-value recall, absent-fact checks, public data.

## Target

Not OCR.

Not one short image.

Not hand-written tiny exact-value prompts.

The target is OMP `snapcompact` architecture:

```text
full prior session history -> serialized archive text -> bitmap frames -> later model input
```

The new renderer must compete with the existing renderer used by:

```text
packages/snapcompact/src/snapcompact.ts
```

## Benchmark contract

Use real or exported OMP session contexts, not synthetic tiny text.

Each benchmark case must keep one canonical source transcript and render it through multiple carriers:

```text
text baseline
current snapcompact renderer
CJK visual-context renderer variants
```

Only carrier changes. Facts, QA, and source transcript stay fixed.

## What must be measured

Primary metrics:

```text
continuation recall F1
task-state exact match
constraint recall exact match
tool-result recall
exact-value recovery
chronology/order accuracy
absent-fact false positive rate
pixels / frame count / image token estimate
```

Text baseline is ceiling. Current `snapcompact` is incumbent. CJK renderer must beat incumbent on CJK-heavy developer history and not collapse on mixed English/code history.

## Input material

Use OMP data/session directories as source material. Do not generate long fake corpora by hand.

Acceptable source material:

```text
real OMP session transcripts
saved compaction archives
tool-heavy debugging sessions
mixed CJK/English coding sessions
long command/result histories
```

Tiny synthetic files may exist only as smoke tests for renderer plumbing. They are not benchmark evidence.

## Required fixture shape

Future fixtures should look like:

```text
fixtures/archives/<case>/
  messages.json        real or exported OMP messages
  serialized.txt       snapcompact-style serialized source
  qa.json              semantic/exact/constraint/absent questions
  metadata.json        source, length, language mix, notes
```

Generated outputs stay under `runs/` and are gitignored. Curated benchmark summaries go under `reports/`.

## Setup

```sh
cargo check
scripts/fetch-fonts.sh
```

Fonts are downloaded on demand. Font binaries are not committed. Sources, versions, hashes, and licenses live in:

```text
fonts/fonts.lock.json
```

Set any OpenAI-compatible vision endpoint:

```sh
export OPENAI_BASE_URL=http://127.0.0.1:4000/v1
export OPENAI_API_KEY=...
export OPENAI_MODEL=google-antigravity/gemini-3.5-flash
```

## Visual preview

Use the local preview UI to tune raster parameters by eye before running model QA:

```sh
bun scripts/preview-server.ts
```

Open:

```text
http://127.0.0.1:8787
```

Controls:

```text
font size
threshold
line spacing
frame size
content width / height
margin
source text
```

The UI renders with the same Rust renderer used by benchmark commands. Outputs go under:

```text
runs/preview/
```

Smoke check:

```sh
bun scripts/preview-server.ts --smoke
```

## Repository layout

```text
src/                  renderer, model calls, scorers
fixtures/             benchmark inputs and small smoke fixtures
matrices/             render parameter grids
scripts/              reproducible entrypoints
fonts/fonts.lock.json font asset lockfile
runs/                 generated results, gitignored
reports/              curated public benchmark summaries
```

## Status

Research prototype. Misleading short synthetic experiments have been removed. Next valid work: build OMP-session archive recall benchmark aligned with `snapcompact`.
