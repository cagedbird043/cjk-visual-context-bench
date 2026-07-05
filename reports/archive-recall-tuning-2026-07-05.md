# Archive Recall Tuning — 2026-07-05

## Scope

Second recall pass after first zpix12 1568px baseline failed. Same real eventized fixture and same QA set. Variables tested:

- native 2000×2000 render
- zpix12 threshold 0.30 vs 0.35
- zpix12 line spacing 0/1/2

No new QA was added. This isolates zpix render effects. Other fonts are intentionally out of scope; prior manual/experimental checks found zpix best for now.

## Previous baseline

| variant | frames | exact/total | strict EM | avg char-F1 |
|---|---:|---:|---:|---:|
| zpix12-t030-l0-f1568 | 2 | 3/12 | 0.2500 | 0.6005 |

## Tuning results

| variant | frames | exact/total | strict EM | avg char-F1 | absent exact |
|---|---:|---:|---:|---:|---:|
| zpix12-t035-l1-f2000 | 2 | 4/12 | 0.3333 | 0.6257 | 2/2 |
| zpix12-t030-l1-f2000 | 2 | 4/12 | 0.3333 | 0.6142 | 2/2 |
| zpix12-t030-l2-f2000 | 2 | 4/12 | 0.3333 | 0.6063 | 2/2 |
| zpix12-t030-l0-f2000 | 1 | 4/12 | 0.3333 | 0.5748 | 2/2 |
| zpix12-t035-l0-f2000 | 1 | 4/12 | 0.3333 | 0.5684 | 2/2 |

## Best variant

`zpix12-t035-l1-f2000` is best in this pass:

- frames: 2
- exact: 4/12
- avg char-F1: 0.6257
- absent exact: 2/2

This improves over 1568 zpix12 but not enough for production.

## Interpretation

Native 2000×2000 helps slightly. Line spacing 1 plus threshold 0.35 is best among tested zpix12 variants. Line spacing 2 does not help more.

Most important result: render tweaks alone do not solve recall. The model still often picks wrong nearby spans or answers with OCR-ish fragments. Density is good, but event image needs better structure/read guidance.

## Decision

Keep `zpix12-t035-l1-f2000` as next visual baseline. Do not treat one-frame dense 2000 render as sufficient. Do not spend more cycles on non-zpix fonts until zpix structure/readability is exhausted.

Next useful change: structure eventized archive with lightweight non-Markdown separators and section anchors, then rerun same QA. Candidate markers:

- turn separator: `｜` or `•`
- speaker markers kept short: `U:`, `A:`
- periodic frame-local anchors: `T001`, `T025`, ...
- topic/event boundaries inserted from user turns, not Markdown headings
