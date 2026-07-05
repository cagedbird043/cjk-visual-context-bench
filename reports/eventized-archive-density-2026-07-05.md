# Eventized Archive Density — 2026-07-05

## Scope

Bitmap archive should store dialogue/event history, not raw execution traces or code tombstones.

Removed from eventized archive:

- thinking blocks
- raw tool calls
- tool result payloads
- raw code blocks
- code omission markers
- long machine/code lines

Kept:

- user messages
- assistant natural-language answers
- decisions, constraints, task state, conclusions
- short exact values already mentioned in dialogue

Code remains in the repository and should be read on demand. Temporary code has no memorization value. Bitmap should preserve event memory.

## Source

Fixture: `fixtures/archives/omp-snapcompact-cjk-window-029/`

- source window: real OMP compaction window
- messages: 896
- user messages: 65
- assistant messages: 396
- tool results in raw log: 435
- tokensBefore: 200998
- raw serialized chars: 579346
- eventized chars: 37070
- eventized dialogue turns: 180

Reduction:

`579346 -> 37070 chars = 15.63× smaller before rendering`

## Frame results

All renders use 1568×1568 frames, zpix, threshold 0.30, line spacing 0, margin 8.

| carrier | variant | frames | compact chars | chars/frame |
|---|---|---:|---:|---:|
| raw-serialized | raw-zpix-12 | 19 | 559190 | 29431.1 |
| raw-serialized | raw-zpix-14 | 26 | 559190 | 21507.3 |
| raw-serialized | raw-zpix-16 | 34 | 559190 | 16446.8 |
| eventized-dialogue | eventized-zpix-12 | 2 | 36891 | 18445.5 |
| eventized-dialogue | eventized-zpix-14 | 3 | 36891 | 12297.0 |
| eventized-dialogue | eventized-zpix-16 | 3 | 36891 | 12297.0 |

## Key comparison

| font size | raw frames | eventized frames | frame reduction |
|---:|---:|---:|---:|
| 12 | 19 | 2 | 9.50× |
| 14 | 26 | 3 | 8.67× |
| 16 | 34 | 3 | 11.33× |

## Interpretation

Raw transcript rendering was wrong benchmark target. It wasted image budget on internal traces a future model cannot use reliably: thinking, tool JSON, code bodies, long outputs, and meaningless `code omitted` markers.

Eventized dialogue archive aligns with intended OMP compression semantics: preserve what happened, what was decided, what remains true, and what the user asked. Do not preserve code as pixels. Do not preserve placeholders for omitted code. Repository files remain source of truth for code.

Best next baseline is `eventized-zpix-14-t030-l0-f1568`: 3 frames for this real compaction window. zpix12 is denser at 2 frames; recall QA must decide whether 12px is readable enough.

## Next benchmark

Use `eventized.txt` for recall QA:

- text baseline over eventized text
- image-only zpix12/14/16
- questions about decisions, constraints, current task state, chronology, absent facts, and exact short values

Density is now plausible. Recall still unproven.
