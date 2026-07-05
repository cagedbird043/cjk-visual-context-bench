# Unicode Snapcompact F1 Benchmark v0.1 — LongBench subset

Generated from `runs/usf1-v0.1/results.json` at 2026-07-05T16:17:26.189Z.

## Summary

Best image preset retains 91.0% of text eligible F1 (0.8753 vs 0.9624).

| image preset | eligible F1 | retention vs text | all-case F1 | frames | chars/frame |
| --- | --- | --- | --- | --- | --- |
| zpix24-binary-2000 | 0.8753 | 91.0% | 0.6228 | 111 | 7667.6 |
| zpix18-half-049-2000 | 0.8132 | 84.5% | 0.6088 | 73 | 11318.6 |

## Leaderboard

| run | model | eligible F1 | eligible cases | avg F1 | correct | exact | semantic | frames | chars/frame |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| text-baseline | google-antigravity/gemini-3.5-flash | 0.9624 | 18/35 | 0.6214 | 14/35 | 8/15 | 6/20 | 0 | 0.0 |
| zpix24-binary-2000 | google-antigravity/gemini-3.5-flash | 0.8753 | 18/35 | 0.6228 | 12/35 | 6/15 | 6/20 | 111 | 7667.6 |
| zpix18-half-049-2000 | google-antigravity/gemini-3.5-flash | 0.8132 | 18/35 | 0.6088 | 10/35 | 6/15 | 4/20 | 73 | 11318.6 |

## Per-task average F1

| task | text-baseline | zpix18-half-049-2000 | zpix24-binary-2000 |
| --- | --- | --- | --- |
| dureader | 0.2639 | 0.1767 | 0.0956 |
| hotpotqa | 0.5967 | 0.6242 | 0.7527 |
| lcc | 0.6488 | 0.4069 | 0.4845 |
| multifieldqa_en | 0.5527 | 0.6728 | 0.6386 |
| multifieldqa_zh | 0.6421 | 0.6655 | 0.5971 |
| passage_retrieval_zh | 1.0000 | 0.9500 | 0.9714 |
| repobench-p | 0.6455 | 0.7654 | 0.8196 |

## Hard / error-analysis bucket

| case | task | text F1 | match kind |
| --- | --- | --- | --- |
| multifieldqa_zh-0002 | multifieldqa_zh | 0.0000 | none |
| multifieldqa_zh-0003 | multifieldqa_zh | 0.2105 | none |
| dureader-0000 | dureader | 0.1053 | none |
| dureader-0001 | dureader | 0.5714 | none |
| dureader-0002 | dureader | 0.1193 | none |
| dureader-0003 | dureader | 0.1899 | none |
| dureader-0004 | dureader | 0.3333 | none |
| multifieldqa_en-0001 | multifieldqa_en | 0.0000 | none |
| multifieldqa_en-0003 | multifieldqa_en | 0.5389 | none |
| multifieldqa_en-0004 | multifieldqa_en | 0.4645 | none |
| hotpotqa-0000 | hotpotqa | 0.4000 | none |
| hotpotqa-0002 | hotpotqa | 0.1667 | none |
| hotpotqa-0004 | hotpotqa | 0.4167 | none |
| lcc-0003 | lcc | 0.1746 | none |
| lcc-0004 | lcc | 0.4516 | none |
| repobench-p-0002 | repobench-p | 0.2821 | none |
| repobench-p-0003 | repobench-p | 0.0000 | none |

## Method

Dataset: deterministic 35-case subset imported from LongBench: 5 cases each from `multifieldqa_zh`, `dureader`, `passage_retrieval_zh`, `multifieldqa_en`, `hotpotqa`, `lcc`, and `repobench-p`.

Carrier modes: text baseline sends `context.txt` as text; image presets render the same `context.txt` into consecutive bitmap frames and send only the frames plus the task prompt.

Main metric: eligible F1. A case is eligible when the text baseline reaches the configured baseline threshold, so visual compression is judged only where the model can solve the underlying text task.

## Caveats

- This is a visual-context compression benchmark, not an OCR transcription benchmark.
- Eligible F1 excludes cases where the text baseline is below 0.7; those cases remain in the hard/error-analysis bucket.
- LongBench semantic tasks are reported F1-first; the conservative `correct` column is diagnostic.
- Image token billing is not claimed here. Frames and chars/frame are reproducible carrier-efficiency proxies.
- Current results use one model endpoint; cross-model claims require reruns with the same dataset and presets.
