# Archive Recall Benchmark — 2026-07-05

## Scope

First recall benchmark for real eventized OMP archive. Same QA set, same fixture, two carriers:

- text baseline over `eventized.txt`
- image-only zpix12 over two 1568×1568 frames

This measures recall, not density.

## Inputs

Fixture: `fixtures/archives/omp-snapcompact-cjk-window-029/`

QA: `fixtures/archives/omp-snapcompact-cjk-window-029/recall_qa.json`

Image carrier: `runs/eventized-render-baseline/eventized-zpix-12-t030-l0-f1568/`

## Summary

| carrier | exact/total | strict EM | avg char-F1 | absent exact |
|---|---:|---:|---:|---:|
| text-baseline | 8/12 | 0.6667 | 0.8092 | 2/2 |
| eventized-zpix12 | 3/12 | 0.2500 | 0.6005 | 2/2 |

## Image failures

| id | score | answer | gold | f1 |
|---|---|---|---|---:|
| quota-refresh-object | semantic | token_quota | 额度 | 0.0000 |
| cache-optimal-rotation | exact | A->B->C->D->A | A-B-C-D-A | 0.8182 |
| fallback-final-decision | semantic | 改 | 不改 | 0.6667 |
| prototype-path | exact | /Projects/CagedBird-Ecosystem/tools/cjk-raster-playground | ~/Projects/CagedBird-Ecosystem/tools/cjk-raster-playground/ | 0.9828 |
| no-vllm-reason | semantic | OpenAI-compatible gateway API runner | OMP 支持中转自身可用模型 | 0.1333 |
| gateway-model | exact | Gemini 3.5 Flash | gemini 3.5 flash | 0.8750 |
| exact-value-correction | semantic | 开发为易最简易处理 不做英文/数字全写过滤。而是分类：Low risk：选目前 Medium risk：可过滤，但观测不能精简成 答案/manifest 最易好文本偏差。报错 不过滤，覆盖 14 选项 真实 OMP 上且 抽样范围 语言。官：所以 视觉强于 渲染 模板 真实 Corpus 应急且落后合作，但测试两种模式：image-only 看画图能读多大 密。Hymn：能打。U：对，做 1 选项。A：那在 2 可打，但 1 不行，对吧？U：对，可以。A：好。U：那如果是这样，就不用搞 14 了，其实就 这几 个。A：对，那如果 实在 选项里，就做 1 个。U：是的。如果做不到，而 不是 “不可确定” | 必须能从图片里恢复，anchor 不能当逃避 | 0.0493 |
| font-size-correction | semantic | 因为我们已经过滤去除了稀疏格式占领了极多空间 | 已经通过去除格式压缩了很多 | 0.5143 |
| public-research-style | semantic | 写代码 | 写论文，直接做实验 | 0.1667 |

## Interpretation

zpix12 eventized images are not recall-ready yet. They preserve enough local text for a few facts, but cross-frame event recall fails on central questions.

Good signs:

- absent/refusal questions passed: 2/2
- `local://cjk-visual-context-plan.md` was read exactly
- prototype path was nearly recovered, but exact mode failed because `~` and trailing slash were lost

Bad signs:

- quota-refresh answer became `token_quota`, not `额度`
- `A-B-C-D-A` became `A->B->C->D->A`, semantically close but exact fail
- fallback final decision flipped from `不改` to `改`
- vLLM reason and exact-value correction were wrong
- image answer sometimes became OCR-like garbage span from wrong region

Text baseline is usable as ceiling but scorer remains strict for verbose semantic answers. Image result is poor enough that scoring looseness does not change the conclusion.

## Conclusion

Density is solved enough for this fixture. Recall is not solved.

Current zpix12 two-frame eventized carrier should not be considered production-ready. Next work should improve render/readability before broad variable search:

1. render native 2000×2000 to match OMP image normalization
2. test zpix12 line spacing 1/2 and threshold 0.35
3. compare Fusion Pixel 12 if available
4. improve event archive structure markers without adding Markdown noise
5. rerun same QA before adding more questions
