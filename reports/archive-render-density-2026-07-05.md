# Archive Render Density Baseline — 2026-07-05

## Scope

Capacity baseline only. This report measures how many 1568px frames are needed to carry one real OMP compaction window. It does not claim recall quality yet.

## Source

Fixture:

`fixtures/archives/omp-snapcompact-cjk-window-029/serialized.txt`

This is a real OMP compaction window, not full session replay and not synthetic short text.

Metadata:

- messages: 896
- user messages: 65
- assistant messages: 396
- tool results: 435
- tokensBefore: 200998
- serialized lines: 14154
- compacted render chars: 559190

The exporter does not fold old compressed archive text back into this source. It models the new window being compacted while old image frames already exist in history.

## Render policy

- frame size: 1568×1568
- font: zpix
- threshold: 0.30
- line spacing: 0
- margin: 8
- whitespace collapsed into dense stream
- structure markers such as `# User ¶`, `# Assistant ¶`, and `<out>` are preserved inline

## Results

| variant | frames | chars/frame | compact chars |
|---|---:|---:|---:|
| zpix-12-t030-l0-f1568 | 19 | 29431.1 | 559190 |
| zpix-14-t030-l0-f1568 | 26 | 21507.3 | 559190 |
| zpix-16-t030-l0-f1568 | 34 | 16446.8 | 559190 |

## Incumbent capacity reference

Current Gemini snapcompact default uses an `8on22-bw` style shape at 1568px. A rough cell-capacity reference is:

`floor(1568 / 8) × floor(1568 / 22) = 196 × 71 = 13916 chars/frame`

For 559190 chars, rough incumbent frame count would be:

`41 frames`

This is only a capacity reference, not exact renderer parity. Existing snapcompact has its own normalization, frame planning, text head/tail, and HQ/LQ shape selection.

## Interpretation

CJK zpix renderer is materially denser at the same 1568px frame budget:

- zpix12: 19 frames, about 2.16× fewer frames than rough incumbent capacity
- zpix14: 26 frames, about 1.58× fewer frames
- zpix16: 34 frames, about 1.21× fewer frames

Next benchmark must test recall over these rendered frames. Density alone is not success.

## Next

Create archive recall QA for this same fixture:

- continuation facts
- task state
- constraints
- exact values
- tool-result facts
- chronology/order
- absent facts

Then compare text baseline vs image-only CJK variants vs current snapcompact renderer.
