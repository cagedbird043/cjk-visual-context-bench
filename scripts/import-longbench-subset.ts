type LongBenchItem = {
  input: string;
  context: string;
  answers: string[];
  length: number;
  dataset: string;
  language: string;
  all_classes: string[] | null;
  _id: string;
};

type OutputCase = {
  id: string;
  source: string;
  source_id: string;
  task: string;
  language: string;
  score: "exact" | "semantic";
  length: number;
  context_chars: number;
  input_chars: number;
  answers: string[];
  paths: {
    context: string;
    qa: string;
    metadata: string;
    source: string;
  };
};

const TASKS = [
  "multifieldqa_zh",
  "dureader",
  "passage_retrieval_zh",
  "multifieldqa_en",
  "hotpotqa",
  "lcc",
  "repobench-p",
];

const EXACT_TASKS: Record<string, true> = {
  passage_retrieval_zh: true,
  lcc: true,
  "repobench-p": true,
};

const EXACT_TASK_NAMES = Object.keys(EXACT_TASKS);

function argValue(name: string, fallback: string): string {
  const index = Bun.argv.indexOf(name);
  return index === -1 ? fallback : Bun.argv[index + 1] ?? fallback;
}

function slugPart(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9_-]+/g, "-").replace(/^-+|-+$/g, "");
}

function caseQuestion(task: string, item: LongBenchItem): string {
  if (task === "lcc" || task === "repobench-p") {
    return [
      "This is a LongBench code completion task.",
      "The archive context is the code prefix/context. Predict only the next missing answer span from that context.",
      "Output the shortest completion span only. No surrounding code. No markdown fences. No explanation.",
      "Task input:",
      item.input || "<empty; complete from archive context>",
    ].join("\n");
  }
  if (task === "passage_retrieval_zh") {
    return [
      "这是 LongBench 中文段落检索任务。请根据下面的任务输入，在 archive context 的候选段落中找出最匹配的段落编号。",
      "只输出段落编号，例如：段落27。不要解释。",
      "任务输入：",
      item.input || "<empty>",
    ].join("\n");
  }
  if (task === "dureader") {
    return [
      "这是 LongBench 中文搜索问答任务。任务输入是搜索查询，请根据 archive context 给出最能回答该查询的简短结论。",
      "不要复述无关剧情细节；只输出答案短语或短句，不要解释。",
      "搜索查询：",
      item.input || "<empty>",
    ].join("\n");
  }
  if (item.language === "zh") {
    return [
      "请只根据 archive context 回答下面的 LongBench 问题。",
      "只输出最短答案片段，不要解释。",
      "问题：",
      item.input || "<empty>",
    ].join("\n");
  }
  return [
    "Answer this LongBench question using only the archive context.",
    "Output only the shortest answer span. No explanation.",
    "Question:",
    item.input || "<empty>",
  ].join("\n");
}

function archiveText(item: LongBenchItem): string {
  return [
    `Source: LongBench`,
    `Dataset: ${item.dataset}`,
    `Language: ${item.language}`,
    `Source id: ${item._id}`,
    "",
    "Input:",
    item.input || "<empty>",
    "",
    "Context:",
    item.context,
  ].join("\n");
}

async function readJsonl(path: string): Promise<LongBenchItem[]> {
  const text = await Bun.file(path).text();
  return text.trim().split("\n").filter(Boolean).map((line) => JSON.parse(line) as LongBenchItem);
}

async function main() {
  const rawDir = argValue("--raw-dir", ".cache/longbench/data");
  const outDir = argValue("--out", "fixtures/longbench/usf1-v0.1-longbench-subset");
  const perTask = Number.parseInt(argValue("--per-task", "5"), 10);
  const cases: OutputCase[] = [];

  for (const task of TASKS) {
    const items = await readJsonl(`${rawDir}/${task}.jsonl`);
    const selected = items.slice(0, perTask);
    for (let index = 0; index < selected.length; index += 1) {
      const item = selected[index];
      const caseId = `${slugPart(task)}-${String(index).padStart(4, "0")}`;
      const caseDir = `${outDir}/cases/${caseId}`;
      const score = EXACT_TASKS[task] ? "exact" : "semantic";
      const context = archiveText(item);
      const qa = [
        {
          id: `${caseId}-answer`,
          score,
          question: caseQuestion(task, item),
          golds: item.answers,
        },
      ];
      const source = {
        ...item,
        imported_from: "LongBench data.zip",
        imported_task: task,
        imported_index: index,
      };
      const metadata = {
        id: caseId,
        source: "LongBench",
        source_url: "https://huggingface.co/datasets/zai-org/LongBench",
        source_id: item._id,
        task,
        language: item.language,
        score,
        length: item.length,
        context_chars: item.context.length,
        input_chars: item.input.length,
        answer_count: item.answers.length,
      };

      await Bun.write(`${caseDir}/context.txt`, context);
      await Bun.write(`${caseDir}/recall_qa.json`, `${JSON.stringify(qa, null, 2)}\n`);
      await Bun.write(`${caseDir}/metadata.json`, `${JSON.stringify(metadata, null, 2)}\n`);
      await Bun.write(`${caseDir}/source.json`, `${JSON.stringify(source, null, 2)}\n`);

      cases.push({
        id: caseId,
        source: "LongBench",
        source_id: item._id,
        task,
        language: item.language,
        score,
        length: item.length,
        context_chars: item.context.length,
        input_chars: item.input.length,
        answers: item.answers,
        paths: {
          context: `cases/${caseId}/context.txt`,
          qa: `cases/${caseId}/recall_qa.json`,
          metadata: `cases/${caseId}/metadata.json`,
          source: `cases/${caseId}/source.json`,
        },
      });
    }
  }

  const manifest = {
    id: "usf1-v0.1-longbench-subset",
    name: "Unicode Snapcompact F1 LongBench Subset v0.1",
    source: "LongBench",
    source_url: "https://huggingface.co/datasets/zai-org/LongBench",
    raw_source: rawDir,
    per_task: perTask,
    case_count: cases.length,
    tasks: TASKS,
    exact_tasks: EXACT_TASK_NAMES,
    generated_at: new Date().toISOString(),
    cases,
  };
  await Bun.write(`${outDir}/manifest.json`, `${JSON.stringify(manifest, null, 2)}\n`);
  await Bun.write(
    `${outDir}/README.md`,
    `# Unicode Snapcompact F1 LongBench Subset v0.1\n\n` +
      `Source: LongBench (<https://huggingface.co/datasets/zai-org/LongBench>).\n\n` +
      `This subset contains ${cases.length} deterministic cases: ${perTask} from each selected task. ` +
      `Each case stores LongBench input and context in \`context.txt\`, one QA item in \`recall_qa.json\`, and source metadata beside it.\n\n` +
      `Exact tasks: ${EXACT_TASK_NAMES.join(", ")}. Other tasks are semantic answer-span tasks.\n`
  );

  console.log(JSON.stringify({ outDir, caseCount: cases.length, tasks: TASKS }, null, 2));
}

await main();
