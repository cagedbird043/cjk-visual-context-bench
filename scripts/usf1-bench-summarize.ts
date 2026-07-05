import * as fs from "node:fs/promises";
import * as path from "node:path";

type QaResult = {
  id: string;
  score: "exact" | "semantic";
  f1: number;
  exact_match: boolean;
  correct?: boolean;
  match_kind?: string;
};

type RunCase = {
  id: string;
  task: string;
  language: string;
  score: "exact" | "semantic";
  output: string;
  renderManifest?: string;
};

type RunManifest = {
  id: string;
  mode: "text" | "image";
  preset: { id: string };
  model: string;
  case_count: number;
  cases: RunCase[];
  dataset: {
    id: string;
    name: string;
  };
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
  exact_match: number;
  semantic_correct: string;
  exact_correct: string;
  avg_f1: number;
  eligible_cases: number;
  eligible_correct: number;
  eligible_avg_f1: number;
  baseline_low_cases: string[];
  total_frames: number;
  avg_frames: number;
  avg_chars_per_frame: number;
  by_task: Record<string, { cases: number; correct: number; avg_f1: number }>;
  case_summaries: CaseSummary[];
};

function argValue(name: string, fallback: string): string {
  const index = Bun.argv.indexOf(name);
  return index === -1 ? fallback : Bun.argv[index + 1] ?? fallback;
}

async function exists(filePath: string): Promise<boolean> {
  return Bun.file(filePath).exists();
}

async function readJsonl(filePath: string): Promise<QaResult[]> {
  const text = await Bun.file(filePath).text();
  return text.trim().split("\n").filter(Boolean).map((line) => JSON.parse(line) as QaResult);
}

async function findRunDirs(root: string): Promise<string[]> {
  if (await exists(path.join(root, "run.json"))) {
    return [root];
  }
  const entries = await fs.readdir(root, { withFileTypes: true });
  const dirs: string[] = [];
  for (const entry of entries) {
    if (!entry.isDirectory()) {
      continue;
    }
    const candidate = path.join(root, entry.name);
    if (await exists(path.join(candidate, "run.json"))) {
      dirs.push(candidate);
    }
  }
  dirs.sort();
  return dirs;
}

async function readFrameStats(renderManifestPath: string | undefined): Promise<{ frames: number; charsPerFrame: number }> {
  if (!renderManifestPath || !(await exists(renderManifestPath))) {
    return { frames: 0, charsPerFrame: 0 };
  }
  const manifest = await Bun.file(renderManifestPath).json();
  return {
    frames: manifest.frame_count ?? 0,
    charsPerFrame: manifest.chars_per_frame ?? 0,
  };
}

function avg(values: number[]): number {
  if (values.length === 0) {
    return 0;
  }
  return values.reduce((sum, value) => sum + value, 0) / values.length;
}

async function summarizeRun(runDir: string): Promise<RunSummary> {
  const manifest = await Bun.file(path.join(runDir, "run.json")).json() as RunManifest;
  const caseSummaries: CaseSummary[] = [];
  for (const item of manifest.cases) {
    const rows = await readJsonl(item.output);
    const result = rows[0];
    if (!result) {
      throw new Error(`missing result row for ${item.id}`);
    }
    const frameStats = await readFrameStats(item.renderManifest);
    caseSummaries.push({
      id: item.id,
      task: item.task,
      language: item.language,
      score: item.score,
      correct: result.correct ?? result.exact_match,
      exact_match: result.exact_match,
      f1: result.f1,
      match_kind: result.match_kind ?? (result.exact_match ? "exact" : "none"),
      frames: frameStats.frames,
      chars_per_frame: frameStats.charsPerFrame,
    });
  }
  const semanticCases = caseSummaries.filter((item) => item.score === "semantic");
  const exactCases = caseSummaries.filter((item) => item.score === "exact");
  const byTask: Record<string, { cases: number; correct: number; avg_f1: number }> = {};
  for (const item of caseSummaries) {
    const group = byTask[item.task] ?? { cases: 0, correct: 0, avg_f1: 0 };
    group.cases += 1;
    if (item.correct) {
      group.correct += 1;
    }
    group.avg_f1 += item.f1;
    byTask[item.task] = group;
  }
  for (const group of Object.values(byTask)) {
    group.avg_f1 = group.cases === 0 ? 0 : group.avg_f1 / group.cases;
  }
  const totalFrames = caseSummaries.reduce((sum, item) => sum + item.frames, 0);
  return {
    id: manifest.id,
    dataset: manifest.dataset.id,
    mode: manifest.mode,
    preset: manifest.preset.id,
    model: manifest.model,
    cases: caseSummaries.length,
    correct: caseSummaries.filter((item) => item.correct).length,
    exact_match: caseSummaries.filter((item) => item.exact_match).length,
    semantic_correct: `${semanticCases.filter((item) => item.correct).length}/${semanticCases.length}`,
    exact_correct: `${exactCases.filter((item) => item.correct).length}/${exactCases.length}`,
    avg_f1: avg(caseSummaries.map((item) => item.f1)),
    eligible_cases: caseSummaries.length,
    eligible_correct: caseSummaries.filter((item) => item.correct).length,
    eligible_avg_f1: avg(caseSummaries.map((item) => item.f1)),
    baseline_low_cases: [],
    total_frames: totalFrames,
    avg_frames: avg(caseSummaries.map((item) => item.frames)),
    avg_chars_per_frame: avg(caseSummaries.filter((item) => item.chars_per_frame > 0).map((item) => item.chars_per_frame)),
    by_task: byTask,
    case_summaries: caseSummaries,
  };
}

function applyBaselineEligibility(summaries: RunSummary[], threshold: number): void {
  const baseline = summaries.find((item) => item.mode === "text");
  if (!baseline) {
    return;
  }
  const baselineById = new Map(baseline.case_summaries.map((item) => [item.id, item.f1]));
  const eligibleIds = new Set(
    baseline.case_summaries.filter((item) => item.f1 >= threshold).map((item) => item.id)
  );
  const lowCases = baseline.case_summaries
    .filter((item) => !eligibleIds.has(item.id))
    .map((item) => item.id);
  for (const summary of summaries) {
    for (const item of summary.case_summaries) {
      item.baseline_f1 = baselineById.get(item.id) ?? 0;
      item.baseline_eligible = eligibleIds.has(item.id);
    }
    const eligible = summary.case_summaries.filter((item) => item.baseline_eligible);
    summary.eligible_cases = eligible.length;
    summary.eligible_correct = eligible.filter((item) => item.correct).length;
    summary.eligible_avg_f1 = avg(eligible.map((item) => item.f1));
    summary.baseline_low_cases = lowCases;
  }
}

function leaderboard(summaries: RunSummary[]): string {
  const sorted = [...summaries].sort((left, right) => {
    if (right.eligible_avg_f1 !== left.eligible_avg_f1) {
      return right.eligible_avg_f1 - left.eligible_avg_f1;
    }
    if (right.avg_f1 !== left.avg_f1) {
      return right.avg_f1 - left.avg_f1;
    }
    return right.correct - left.correct;
  });
  const lines = [
    "# Unicode Snapcompact F1 leaderboard",
    "",
    "| run | mode | preset | model | eligible F1 | eligible cases | avg F1 | correct | exact | semantic | frames | chars/frame |",
    "|---|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|",
  ];
  for (const item of sorted) {
    lines.push(
      `| ${item.id} | ${item.mode} | ${item.preset} | ${item.model} | ${item.eligible_avg_f1.toFixed(4)} | ${item.eligible_cases}/${item.cases} | ${item.avg_f1.toFixed(4)} | ${item.correct}/${item.cases} | ${item.exact_correct} | ${item.semantic_correct} | ${item.total_frames} | ${item.avg_chars_per_frame.toFixed(1)} |`
    );
  }
  lines.push("");
  return `${lines.join("\n")}\n`;
}

async function main() {
  const root = argValue("--run-dir", "runs/usf1");
  const baselineThreshold = Number.parseFloat(argValue("--baseline-threshold", "0.7"));
  const outDir = argValue("--out", root);
  const runDirs = await findRunDirs(root);
  if (runDirs.length === 0) {
    throw new Error(`no run.json found under ${root}`);
  }
  const summaries = [];
  for (const dir of runDirs) {
    summaries.push(await summarizeRun(dir));
  }
  applyBaselineEligibility(summaries, baselineThreshold);
  await fs.mkdir(outDir, { recursive: true });
  await Bun.write(path.join(outDir, "results.json"), `${JSON.stringify({ generated_at: new Date().toISOString(), baseline_threshold: baselineThreshold, runs: summaries }, null, 2)}\n`);
  await Bun.write(path.join(outDir, "leaderboard.md"), leaderboard(summaries));
  console.log(JSON.stringify({ runCount: summaries.length, outDir }, null, 2));
}

await main();
