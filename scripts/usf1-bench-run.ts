import * as fs from "node:fs/promises";
import * as path from "node:path";

type DatasetCase = {
  id: string;
  task: string;
  language: string;
  score: "exact" | "semantic";
  paths: {
    context: string;
    qa: string;
    metadata: string;
    source: string;
  };
};

type DatasetManifest = {
  id: string;
  name: string;
  cases: DatasetCase[];
};

type Preset = {
  id: string;
  font: string;
  fontSize: number;
  threshold: number;
  lineSpacing: number;
  frameSize: number;
  margin: number;
};

type RunCase = {
  id: string;
  task: string;
  language: string;
  score: "exact" | "semantic";
  context: string;
  qa: string;
  output: string;
  render?: string;
  renderManifest?: string;
};

const PRESETS: Record<string, Preset> = {
  "zpix24-binary-2000": {
    id: "zpix24-binary-2000",
    font: "fonts/zpix.ttf",
    fontSize: 24,
    threshold: 0.49,
    lineSpacing: 2,
    frameSize: 2000,
    margin: 8,
  },
  "zpix18-half-049-2000": {
    id: "zpix18-half-049-2000",
    font: "fonts/zpix.ttf",
    fontSize: 18,
    threshold: 0.49,
    lineSpacing: 2,
    frameSize: 2000,
    margin: 8,
  },
};

function argValue(name: string, fallback: string): string {
  const index = Bun.argv.indexOf(name);
  return index === -1 ? fallback : Bun.argv[index + 1] ?? fallback;
}

function hasFlag(name: string): boolean {
  return Bun.argv.includes(name);
}

function slug(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9._-]+/g, "-").replace(/^-+|-+$/g, "");
}

async function exists(filePath: string): Promise<boolean> {
  return Bun.file(filePath).exists();
}

async function runCommand(cmd: string[], env: Record<string, string | undefined>): Promise<{ stdout: string; stderr: string }> {
  const proc = Bun.spawn(cmd, {
    stdout: "pipe",
    stderr: "pipe",
    env: { ...Bun.env, ...env },
  });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  if (exitCode !== 0) {
    throw new Error(`${cmd.join(" ")}\nexit=${exitCode}\nstdout=${stdout}\nstderr=${stderr}`);
  }
  return { stdout, stderr };
}

async function evalEnv(modelArg: string, useOmpGateway: boolean): Promise<Record<string, string>> {
  const env: Record<string, string> = {};
  if (modelArg) {
    env.OPENAI_MODEL = modelArg;
  }
  if (useOmpGateway) {
    env.OPENAI_BASE_URL = argValue("--base-url", "http://127.0.0.1:4000/v1");
    const token = await runCommand(["omp", "auth-gateway", "token"], {});
    env.OPENAI_API_KEY = token.stdout.trim();
  }
  return env;
}

function selectCases(manifest: DatasetManifest): DatasetCase[] {
  const caseIds = argValue("--cases", "");
  if (caseIds) {
    const wanted = new Set(caseIds.split(",").map((id) => id.trim()).filter(Boolean));
    return manifest.cases.filter((item) => wanted.has(item.id));
  }
  if (hasFlag("--one-per-task")) {
    const seen = new Set<string>();
    return manifest.cases.filter((item) => {
      if (seen.has(item.task)) {
        return false;
      }
      seen.add(item.task);
      return true;
    });
  }
  const limit = Number.parseInt(argValue("--limit", "0"), 10);
  return limit > 0 ? manifest.cases.slice(0, limit) : manifest.cases;
}

async function main() {
  const manifestPath = argValue("--manifest", "fixtures/longbench/usf1-v0.1-longbench-subset/manifest.json");
  const mode = argValue("--mode", "image");
  if (mode !== "text" && mode !== "image") {
    throw new Error("--mode must be text or image");
  }
  const presetId = argValue("--preset", mode === "image" ? "zpix24-binary-2000" : "text-baseline");
  const preset = mode === "image" ? PRESETS[presetId] : null;
  if (mode === "image" && !preset) {
    throw new Error(`unknown preset ${presetId}; known presets: ${Object.keys(PRESETS).join(", ")}`);
  }
  const model = argValue("--model", Bun.env.OPENAI_MODEL ?? "");
  const outRoot = argValue("--out", "runs/usf1");
  const maxTokens = argValue("--max-tokens", "160");
  const manifest = await Bun.file(manifestPath).json() as DatasetManifest;
  const manifestDir = path.dirname(manifestPath);
  const selected = selectCases(manifest);
  if (selected.length === 0) {
    throw new Error("no cases selected");
  }

  const runIdParts = [manifest.id, mode === "text" ? "text-baseline" : presetId, model ? slug(model) : "env-model"];
  if (hasFlag("--one-per-task")) {
    runIdParts.push("one-per-task");
  } else if (argValue("--limit", "0") !== "0") {
    runIdParts.push(`limit-${argValue("--limit", "0")}`);
  }
  const runId = runIdParts.join("__");
  const runDir = path.join(outRoot, runId);
  await fs.mkdir(path.join(runDir, "cases"), { recursive: true });

  const binaryPath = argValue("--binary", "target/release/cjk-visual-context-bench");
  if (!(await exists(binaryPath))) {
    console.log("building release binary");
    await runCommand(["cargo", "build", "--release"], {});
  }
  const env = await evalEnv(model, hasFlag("--omp-gateway"));
  if (!env.OPENAI_MODEL && !Bun.env.OPENAI_MODEL) {
    throw new Error("missing OPENAI_MODEL; pass --model or set env");
  }

  const runCases: RunCase[] = [];
  for (const item of selected) {
    const caseDir = path.join(runDir, "cases", item.id);
    await fs.mkdir(caseDir, { recursive: true });
    const context = path.join(manifestDir, item.paths.context);
    const qa = path.join(manifestDir, item.paths.qa);
    const output = path.join(caseDir, "answers.jsonl");
    const runCase: RunCase = {
      id: item.id,
      task: item.task,
      language: item.language,
      score: item.score,
      context,
      qa,
      output,
    };
    console.log(`case ${item.id}: ${mode}`);
    if (mode === "image" && preset) {
      const renderDir = path.join(caseDir, "frames");
      await runCommand([
        binaryPath,
        "render-archive",
        "--text", context,
        "--out-dir", renderDir,
        "--name", `${item.id}-${preset.id}`,
        "--font", preset.font,
        "--font-size", String(preset.fontSize),
        "--threshold", String(preset.threshold),
        "--line-spacing", String(preset.lineSpacing),
        "--frame-size", String(preset.frameSize),
        "--margin", String(preset.margin),
      ], {});
      runCase.render = renderDir;
      runCase.renderManifest = path.join(renderDir, "manifest.json");
      await runCommand([
        binaryPath,
        "archive-qa",
        "--images-dir", renderDir,
        "--qa", qa,
        "--out", output,
        "--max-tokens", maxTokens,
      ], env);
    } else {
      await runCommand([
        binaryPath,
        "archive-qa",
        "--text", context,
        "--qa", qa,
        "--out", output,
        "--max-tokens", maxTokens,
      ], env);
    }
    runCases.push(runCase);
  }

  const runManifest = {
    id: runId,
    dataset: {
      id: manifest.id,
      name: manifest.name,
      manifest: manifestPath,
    },
    mode,
    preset: preset ?? { id: "text-baseline" },
    model: model || Bun.env.OPENAI_MODEL || "env-model",
    max_tokens: Number.parseInt(maxTokens, 10),
    case_count: runCases.length,
    generated_at: new Date().toISOString(),
    cases: runCases,
  };
  await Bun.write(path.join(runDir, "run.json"), `${JSON.stringify(runManifest, null, 2)}\n`);
  console.log(JSON.stringify({ runDir, caseCount: runCases.length }, null, 2));
}

await main();
