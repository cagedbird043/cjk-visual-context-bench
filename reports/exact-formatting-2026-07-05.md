# Exact Formatting Benchmark — 2026-07-05

## Question

Can image-internal formatting improve exact-value recovery for dense CJK visual-context bitmaps without external anchors?

## Setup

- Model: `google-antigravity/gemini-3.5-flash` via OMP auth-gateway.
- Renderer: compact stream, no layout preservation.
- Font: zpix.
- Scoring: `score = exact`, trim-only comparison; one wrong character fails.
- QA items: candidate ID, balanced ID, model ID, Chinese exact phrase, file path, sha256.

## Main result

`annotated + zpix-14-t030-l0-w1024` is the first stable 100% exact-value point.

It preserves the original exact strings in the image and adds local disambiguation text:

`l0 is lowercase ell + digit zero; w750 is double-u + 750`.

This is not an external anchor. The image still contains the exact value and the disambiguation rule.

## Best one-shot rows

| format | variant | pixels | exact EM | exact F1 |
|---|---:|---:|---:|---:|
| escaped | zpix-12-t030-l0-w750 | 65790 | 1.0000 | 1.0000 |
| segmented | zpix-14-t030-l0-w1024 | 70652 | 1.0000 | 1.0000 |
| annotated | zpix-14-t030-l0-w1024 | 85198 | 1.0000 | 1.0000 |

## Retest

| format | variant | run | exact EM | failures |
|---|---|---:|---:|---|
| annotated | zpix-14-t030-l0-w1024 | 1 | 1.0000 |  |
| annotated | zpix-14-t030-l0-w1024 | 2 | 1.0000 |  |
| annotated | zpix-14-t030-l0-w1024 | 3 | 1.0000 |  |
| escaped | zpix-12-t030-l0-w750 | 1 | 0.6667 | candidate-id=zpix-{one}2-t{zero}3{zero}-{ell}{zero}-{w}75{zero}; balanced-id=zpix-{one}4-t{zero}3{zero}-{ell}{zero}-{w}{one}{zero}24 |
| escaped | zpix-12-t030-l0-w750 | 2 | 1.0000 |  |
| escaped | zpix-12-t030-l0-w750 | 3 | 0.8333 | candidate-id=zpix-{one}2-t{zero}3{zero}-{ell}{zero}-{w}75{zero} |

## Interpretation

Plain labels, brackets, quotes, and global glyph legends did not fix `l0 -> 10`. They stayed at 66.67% exact EM on this six-item exact-value set.

Segmenting with spaces around hyphens helped once, but retest exposed instability: `zpix-12` became `zpix-l2` in 2/3 retests. It moved the ambiguity from `l0/10` to `12/l2`.

Escaped tokens can work, but are unstable because the model sometimes returns the encoded string instead of decoding it. That makes custom escape syntax risky unless the QA prompt/protocol is changed.

Inline local annotation is currently best: it keeps the exact original string visible and teaches only the ambiguous segment beside it.

## Current engineering point

`annotated + zpix-14 + width 1024` is the next baseline for exact-value research.

Measured image: 1039×82 = 85,198 pixels.

This is still smaller than the earlier `zpix16-w1024` exact-dev-context image at 225,463 pixels, while passing this targeted exact-formatting suite.

## Next experiment

Run a larger annotated corpus with multiple ambiguous patterns:

- `l0` vs `10`
- `12` vs `l2`
- `w` vs `v`
- `O` vs `0`
- `I` vs `1`
- `3.5` vs `1.5`
- `帧` vs `频`

Gate for the next milestone: exact EM ≥ 95% over a larger suite, then add absent/hallucination tests.
