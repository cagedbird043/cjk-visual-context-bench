# Structured Archive Recall — 2026-07-05

## Scope

Test whether lightweight non-Markdown turn anchors improve image recall. Same fixture, same QA, same render settings.

Baseline carrier:

`eventized.txt -> zpix12-t035-l1-f2000`

Structured carrier:

`eventized_structured.txt -> structured-zpix12-t035-l1-f2000`

Structure added:

- header: `ARCHIVE EVENT STREAM`
- turn anchors: `T001`, `T002`, ...
- speaker markers: `U:`, `A:`
- turn separator: `｜`

No Markdown headings, raw tool traces, thinking blocks, code, or code tombstones.

## Results

| variant | frames | compact chars | exact/total | strict EM | avg char-F1 | absent exact |
|---|---:|---:|---:|---:|---:|---:|
| unstructured-zpix12-t035-l1-f2000 | 2 | 35460 | 4/12 | 0.3333 | 0.6257 | 2/2 |
| structured-zpix12-t035-l1-f2000 | 2 | 36741 | 2/12 | 0.1667 | 0.5024 | 2/2 |

## Structured failures

| id | answer | gold | f1 |
|---|---|---|---:|
| quota-refresh-object | 缓存 | 额度 | 0.0000 |
| cache-optimal-rotation | A-B-C-D-E | A-B-C-D-A | 0.8889 |
| fallback-final-decision | 改 | 不改 | 0.6667 |
| plan-storage-uri | local://engineering-balance | local://cjk-visual-context-plan.md | 0.4918 |
| prototype-path | /Projects/CagedBird-ecosystems/cjk-raster-playground | ~/Projects/CagedBird-Ecosystem/tools/cjk-raster-playground/ | 0.9189 |
| no-vllm-reason | 因为已经实现了本地的 evaluation，可以自己跑评估，不需要依赖 vLLM | OMP自己能当凭证中转和模型代理 | 0.1111 |
| gateway-model | openai-codex | gemini 3.5 flash | 0.2143 |
| exact-value-correction | 直接报错 | 必须能从图片里恢复，anchor 不能当逃避 | 0.0000 |
| font-size-correction | 因为它可以能够更便宜 | 已经通过去除格式压缩了很多 | 0.0000 |
| public-research-style | 直接跑实验，发数据 | 直接做实验，公开数据 | 0.7368 |

## Interpretation

Naive anchors did not help. They increased compact chars from 35460 to 36741 and reduced recall quality.

The model still answered from wrong nearby regions. The added `Txxx` markers may add visual clutter without giving the model a useful retrieval strategy.

Decision: do not keep this structured format as baseline. Keep unstructured zpix12-t035-l1-f2000 as current best measured variant. Next work should not add more visual markers blindly. Better next candidate is layout/readability change: fewer chars per frame via larger effective line spacing or more frames, while preserving plain event text.
