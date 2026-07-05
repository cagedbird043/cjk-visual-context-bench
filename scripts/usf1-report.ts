import * as fs from "node:fs/promises";
import * as path from "node:path";

type TaskStats = {
  cases: number;
  correct: number;
  avg_f1: number;
};

type CaseSummary = {
  id: string;
  task: string;
  language: string;
  score: "exact" | "semantic";
  correct: boolean;
  exact_match: boolean;
  f1: number;
  match_kind: string;
  frames: number;
  chars_per_frame: number;
  baseline_f1?: number;
  baseline_eligible?: boolean;
};

type RunSummary = {
  id: string;
  dataset: string;
  mode: string;
  preset: string;
  model: string;
  cases: number;
  correct: number;
  exact_correct: string;
  semantic_correct: string;
  avg_f1: number;
  eligible_cases: number;
  eligible_avg_f1: number;
  baseline_low_cases: string[];
  total_frames: number;
  avg_frames: number;
  avg_chars_per_frame: number;
  by_task: Record<string, TaskStats>;
  case_summaries: CaseSummary[];
};

type Results = {
  generated_at: string;
  baseline_threshold: number;
  runs: RunSummary[];
};

function argValue(name: string, fallback: string): string {
  const index = Bun.argv.indexOf(name);
  return index === -1 ? fallback : Bun.argv[index + 1] ?? fallback;
}

function fmt(value: number): string {
  return value.toFixed(4);
}

function pct(value: number): string {
  return `${(value * 100).toFixed(1)}%`;
}

function ratio(numerator: number, denominator: number): string {
  return denominator === 0 ? "0.0%" : pct(numerator / denominator);
}

function table(headers: string[], rows: string[][]): string {
  return [
    `| ${headers.join(" | ")} |`,
    `| ${headers.map(() => "---").join(" | ")} |`,
    ...rows.map((row) => `| ${row.join(" | ")} |`),
  ].join("\n");
}

function runLabel(run: RunSummary): string {
  return run.mode === "text" ? "text-baseline" : run.preset;
}

function taskNames(runs: RunSummary[]): string[] {
  const names = new Set<string>();
  for (const run of runs) {
    for (const name of Object.keys(run.by_task)) {
      names.add(name);
    }
  }
  return [...names].sort();
}

function findTextRun(results: Results): RunSummary | undefined {
  return results.runs.find((run) => run.mode === "text");
}

function imageRuns(results: Results): RunSummary[] {
  return results.runs.filter((run) => run.mode === "image");
}

function retentionRows(results: Results): string[][] {
  const text = findTextRun(results);
  if (!text) {
    return [];
  }
  return imageRuns(results)
    .sort((left, right) => right.eligible_avg_f1 - left.eligible_avg_f1 || right.avg_f1 - left.avg_f1)
    .map((run) => [
      run.preset,
      fmt(run.eligible_avg_f1),
      pct(run.eligible_avg_f1 / text.eligible_avg_f1),
      fmt(run.avg_f1),
      String(run.total_frames),
      run.avg_chars_per_frame.toFixed(1),
    ]);
}

function leaderboardRows(results: Results): string[][] {
  return [...results.runs]
    .sort((left, right) => right.eligible_avg_f1 - left.eligible_avg_f1 || right.avg_f1 - left.avg_f1)
    .map((run) => [
      runLabel(run),
      run.model,
      fmt(run.eligible_avg_f1),
      `${run.eligible_cases}/${run.cases}`,
      fmt(run.avg_f1),
      `${run.correct}/${run.cases}`,
      run.exact_correct,
      run.semantic_correct,
      String(run.total_frames),
      run.avg_chars_per_frame.toFixed(1),
    ]);
}

function perTaskRows(results: Results): string[][] {
  const names = taskNames(results.runs);
  return names.map((name) => {
    const row = [name];
    for (const run of results.runs) {
      const stats = run.by_task[name];
      row.push(stats ? fmt(stats.avg_f1) : "—");
    }
    return row;
  });
}

function hardBucketRows(text: RunSummary | undefined): string[][] {
  if (!text) {
    return [];
  }
  return text.case_summaries
    .filter((item) => !item.baseline_eligible)
    .map((item) => [item.id, item.task, fmt(item.f1), item.match_kind]);
}

function caveats(results: Results): string {
  return [
    "- This is a visual-context compression benchmark, not an OCR transcription benchmark.",
    `- Eligible F1 excludes cases where the text baseline is below ${results.baseline_threshold}; those cases remain in the hard/error-analysis bucket.`,
    "- LongBench semantic tasks are reported F1-first; the conservative `correct` column is diagnostic.",
    "- Image token billing is not claimed here. Frames and chars/frame are reproducible carrier-efficiency proxies.",
    "- Current results use one model endpoint; cross-model claims require reruns with the same dataset and presets.",
  ].join("\n");
}

async function main() {
  const resultsPath = argValue("--results", "runs/usf1-v0.1/results.json");
  const outPath = argValue("--out", "reports/usf1-v0.1-longbench.md");
  const results = await Bun.file(resultsPath).json() as Results;
  const text = findTextRun(results);
  const retentions = retentionRows(results);
  const bestImage = imageRuns(results).sort((left, right) => right.eligible_avg_f1 - left.eligible_avg_f1)[0];
  const retentionLine = text && bestImage
    ? `Best image preset retains ${pct(bestImage.eligible_avg_f1 / text.eligible_avg_f1)} of text eligible F1 (${fmt(bestImage.eligible_avg_f1)} vs ${fmt(text.eligible_avg_f1)}).`
    : "No image-vs-text retention computed.";

  const markdown = [
    "# Unicode Snapcompact F1 Benchmark v0.1 — LongBench subset",
    "",
    `Generated from \`${resultsPath}\` at ${results.generated_at}.`,
    "",
    "## Summary",
    "",
    retentionLine,
    "",
    retentions.length > 0
      ? table(["image preset", "eligible F1", "retention vs text", "all-case F1", "frames", "chars/frame"], retentions)
      : "No image runs found.",
    "",
    "## Leaderboard",
    "",
    table(["run", "model", "eligible F1", "eligible cases", "avg F1", "correct", "exact", "semantic", "frames", "chars/frame"], leaderboardRows(results)),
    "",
    "## Per-task average F1",
    "",
    table(["task", ...results.runs.map(runLabel)], perTaskRows(results)),
    "",
    "## Hard / error-analysis bucket",
    "",
    table(["case", "task", "text F1", "match kind"], hardBucketRows(text)),
    "",
    "## Method",
    "",
    "Dataset: deterministic 35-case subset imported from LongBench: 5 cases each from `multifieldqa_zh`, `dureader`, `passage_retrieval_zh`, `multifieldqa_en`, `hotpotqa`, `lcc`, and `repobench-p`.",
    "",
    "Carrier modes: text baseline sends `context.txt` as text; image presets render the same `context.txt` into consecutive bitmap frames and send only the frames plus the task prompt.",
    "",
    "Main metric: eligible F1. A case is eligible when the text baseline reaches the configured baseline threshold, so visual compression is judged only where the model can solve the underlying text task.",
    "",
    "## Caveats",
    "",
    caveats(results),
    "",
  ].join("\n");

  await fs.mkdir(path.dirname(outPath), { recursive: true });
  await Bun.write(outPath, markdown);
  console.log(JSON.stringify({ outPath }, null, 2));
}

await main();
