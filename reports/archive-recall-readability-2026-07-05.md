# Archive Recall Readability Budget — 2026-07-05

## Scope

Test whether reducing usable content area inside fixed 2000×2000 frames improves recall. Same eventized fixture, same QA, same font and threshold baseline.

Render baseline before this pass:

- variant: `zpix12-t035-l1-f2000`
- content area: full frame
- frames: 2
- exact: 4/12
- avg char-F1: 0.6257

New variable: `--content-width` / `--content-height`, with fixed output frame still 2000×2000. Last frame remains square.

## Results

| variant | content area | frames | exact/total | strict EM | avg char-F1 | absent exact |
|---|---:|---:|---:|---:|---:|---:|
| zpix12-t035-l1-f2000 | full 2000×2000 | 2 | 4/12 | 0.3333 | 0.6257 | 2/2 |
| zpix12-t035-l1-f2000-c1400 | 1400×1400 | 3 | 4/12 | 0.3333 | 0.6287 | 2/2 |
| zpix12-t035-l1-f2000-c1800 | 1800×1800 | 2 | 4/12 | 0.3333 | 0.6254 | 2/2 |
| zpix12-t035-l1-f2000-c1600 | 1600×1600 | 2 | 2/12 | 0.1667 | 0.4645 | 2/2 |

## Best in this pass

`zpix12-t035-l1-f2000-c1400`:

- content area: 1400×1400
- frames: 3
- exact: 4/12
- avg char-F1: 0.6287
- absent exact: 2/2

## Interpretation

Reducing density helps only marginally. The 1400×1400 content box increases average F1 slightly but keeps strict EM at 4/12 and costs one extra frame. The 1600×1600 box is worse.

Conclusion: simple whitespace/content-box reduction is not enough. The model still has trouble retrieving the correct event span from dense archive images.

Keep `zpix12-t035-l1-f2000` as current baseline unless future repeats show 1400×1400 is stable. Do not switch baseline based on a tiny F1 delta.

Next useful direction: split event archive by semantic time windows before rendering, so each frame has a clearer local topic. That is different from adding noisy anchors; it changes page organization.
