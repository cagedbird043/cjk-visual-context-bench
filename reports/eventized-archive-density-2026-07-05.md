# Eventized Archive Density — 2026-07-05

## Scope

Bitmap archive should store plain dialogue/event history, not raw execution traces, code tombstones, or Markdown syntax noise.

Removed from eventized archive:

- thinking blocks
- raw tool calls
- tool result payloads
- raw code blocks
- code omission markers
- long machine/code lines
- Markdown presentation syntax: headings, bullets, quote markers, emphasis markers, inline code backticks, markdown links

Kept:

- user messages
- assistant natural-language answers
- decisions, constraints, task state, conclusions
- short exact values already mentioned in dialogue

Code remains in the repository and should be read on demand. Temporary code has no memorization value. Markdown formatting does not render in bitmap and mostly harms density/readability. Bitmap should preserve plain event memory.

## Source

Fixture: `fixtures/archives/omp-snapcompact-cjk-window-029/`

- source window: real OMP compaction window
- messages: 896
- user messages: 65
- assistant messages: 396
- tool results in raw log: 435
- tokensBefore: 200998
- raw serialized chars: 579346
- eventized chars: 35639
- eventized dialogue turns: 180

Reduction:

`579346 -> 35639 chars = 16.26× smaller before rendering`

## Frame results

All renders use 1568×1568 frames, zpix, threshold 0.30, line spacing 0, margin 8.

| carrier | variant | frames | compact chars | chars/frame |
|---|---|---:|---:|---:|
| raw-serialized | raw-zpix-12 | 19 | 559190 | 29431.1 |
| raw-serialized | raw-zpix-14 | 26 | 559190 | 21507.3 |
| raw-serialized | raw-zpix-16 | 34 | 559190 | 16446.8 |
| eventized-dialogue | eventized-zpix-12 | 2 | 35460 | 17730.0 |
| eventized-dialogue | eventized-zpix-14 | 3 | 35460 | 11820.0 |
| eventized-dialogue | eventized-zpix-16 | 3 | 35460 | 11820.0 |

## Key comparison

| font size | raw frames | eventized frames | frame reduction |
|---:|---:|---:|---:|
| 12 | 19 | 2 | 9.50× |
| 14 | 26 | 3 | 8.67× |
| 16 | 34 | 3 | 11.33× |

## Interpretation

Raw transcript rendering was wrong benchmark target. It wasted image budget on internal traces a future model cannot use reliably: thinking, tool JSON, code bodies, long outputs, meaningless `code omitted` markers, and Markdown formatting characters.

Eventized dialogue archive aligns with intended OMP compression semantics: preserve what happened, what was decided, what remains true, and what the user asked. Do not preserve code as pixels. Do not preserve placeholders for omitted code. Do not preserve Markdown syntax that is not visually rendered. Repository files remain source of truth for code.

Best next baseline is `eventized-zpix-12-t030-l0-f1568`: 2 frames for this real compaction window and the sharpest observed zpix rendering. zpix14 and zpix16 are density/legibility controls, not defaults.

## Next benchmark

Use `eventized.txt` for recall QA:

- text baseline over eventized text
- image-only zpix12/14/16
- questions about decisions, constraints, current task state, chronology, absent facts, and exact short values

Density is now plausible. Recall still unproven.
