import * as fs from "node:fs/promises";
import * as path from "node:path";

type RunSummary = {
  mode: string;
  preset: string;
  model: string;
  eligible_avg_f1: number;
  avg_f1: number;
  eligible_cases: number;
  cases: number;
  total_frames: number;
  avg_chars_per_frame: number;
};

type Results = {
  baseline_threshold: number;
  runs: RunSummary[];
};

function argValue(name: string, fallback: string): string {
  const index = Bun.argv.indexOf(name);
  return index === -1 ? fallback : Bun.argv[index + 1] ?? fallback;
}

function pct(value: number): string {
  return `${(value * 100).toFixed(1)}%`;
}

function esc(value: string): string {
  return value.replace(/[&<>]/g, (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;" })[char] ?? char);
}

async function main() {
  const resultsPath = argValue("--results", "runs/usf1-v0.1/results.json");
  const outPath = argValue("--out", "reports/usf1-v0.1-frontier.svg");
  const results = await Bun.file(resultsPath).json() as Results;
  const text = results.runs.find((run) => run.mode === "text");
  const z18 = results.runs.find((run) => run.preset === "zpix18-half-049-2000");
  const z24 = results.runs.find((run) => run.preset === "zpix24-binary-2000");
  if (!text || !z18 || !z24) {
    throw new Error("results must include text baseline, zpix18, and zpix24 runs");
  }

  const width = 1600;
  const height = 900;
  const margin = { left: 140, right: 140, top: 235, bottom: 190 };
  const plotW = width - margin.left - margin.right;
  const plotH = height - margin.top - margin.bottom;
  const xMin = 60;
  const xMax = 120;
  const yMin = 0.78;
  const yMax = 0.98;
  const x = (value: number) => margin.left + ((value - xMin) / (xMax - xMin)) * plotW;
  const y = (value: number) => margin.top + (1 - (value - yMin) / (yMax - yMin)) * plotH;
  const ticksX = [60, 70, 80, 90, 100, 110, 120];
  const ticksY = [0.8, 0.84, 0.88, 0.92, 0.96];
  const point = (run: RunSummary, color: string, label: string, dx: number, dy: number) => {
    const px = x(run.total_frames);
    const py = y(run.eligible_avg_f1);
    const retention = run.eligible_avg_f1 / text.eligible_avg_f1;
    return `
      <circle cx="${px.toFixed(1)}" cy="${py.toFixed(1)}" r="11" fill="${color}" stroke="#fff" stroke-width="4"/>
      <circle cx="${px.toFixed(1)}" cy="${py.toFixed(1)}" r="22" fill="none" stroke="${color}" stroke-opacity="0.18" stroke-width="10"/>
      <text x="${(px + dx).toFixed(1)}" y="${(py + dy).toFixed(1)}" class="point-label" fill="${color}">${esc(label)}</text>
      <text x="${(px + dx).toFixed(1)}" y="${(py + dy + 40).toFixed(1)}" class="point-sub">${pct(retention)} text F1 · ${run.total_frames} frames</text>
      <text x="${(px + dx).toFixed(1)}" y="${(py + dy + 72).toFixed(1)}" class="point-sub">${run.avg_chars_per_frame.toFixed(0)} chars/frame</text>
    `;
  };
  const frontierLine = `M ${x(z18.total_frames).toFixed(1)} ${y(z18.eligible_avg_f1).toFixed(1)} L ${x(z24.total_frames).toFixed(1)} ${y(z24.eligible_avg_f1).toFixed(1)}`;

  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" role="img" aria-labelledby="title desc">
  <title id="title">Unicode Snapcompact F1 v0.1 preset frontier</title>
  <desc id="desc">Benchmark chart comparing zpix18 and zpix24 visual context presets by total frames and eligible F1 on the LongBench subset with Gemini 3.5 Flash.</desc>
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#f7fbff"/><stop offset="0.58" stop-color="#ffffff"/><stop offset="1" stop-color="#f5f0ff"/></linearGradient>
    <linearGradient id="frontier" x1="0" y1="0" x2="1" y2="0"><stop offset="0" stop-color="#7c3aed"/><stop offset="1" stop-color="#0f4bd8"/></linearGradient>
    <filter id="softShadow" x="-20%" y="-20%" width="140%" height="140%"><feDropShadow dx="0" dy="16" stdDeviation="18" flood-color="#163568" flood-opacity="0.14"/></filter>
    <style>
      text { font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; }
      .kicker { font-size: 21px; font-weight: 800; letter-spacing: .01em; fill: #0b163f; }
      .title { font-size: 54px; font-weight: 900; letter-spacing: -0.045em; fill: #08132f; }
      .subtitle { font-size: 23px; fill: #566176; font-weight: 560; }
      .axis { stroke: #17213a; stroke-width: 2.4; }
      .grid { stroke: #dbe2ef; stroke-width: 1.2; }
      .tick { fill: #5c667a; font-size: 20px; }
      .axis-label { fill: #17213a; font-size: 25px; font-weight: 800; }
      .point-label { font-size: 33px; font-weight: 900; letter-spacing: -0.025em; }
      .point-sub { font-size: 20px; fill: #566176; font-weight: 700; }
      .ceiling { stroke: #f59e0b; stroke-width: 3.5; stroke-dasharray: 12 9; }
      .ceiling-text { fill: #b45309; font-size: 21px; font-weight: 850; }
      .card-title { fill: #fff; font-size: 27px; font-weight: 900; }
      .card-line { fill: #c7d2fe; font-size: 20px; font-weight: 760; }
      .small { font-size: 17px; fill: #6b7280; }
    </style>
  </defs>
  <rect width="${width}" height="${height}" rx="40" fill="url(#bg)"/>
  <rect x="34" y="34" width="${width - 68}" height="${height - 68}" rx="34" fill="#ffffff" opacity="0.76" filter="url(#softShadow)"/>

  <text x="90" y="82" class="kicker">USF1 v0.1 · LongBench subset · Gemini 3.5 Flash</text>
  <text x="90" y="150" class="title">Unicode Snapcompact F1 preset frontier</text>
  <text x="90" y="205" class="subtitle">Higher is better. Left is denser. zpix24 trades 52% more frames for 91.0% text-baseline eligible F1.</text>

  ${ticksX.map((tick) => `<line x1="${x(tick).toFixed(1)}" y1="${margin.top}" x2="${x(tick).toFixed(1)}" y2="${height - margin.bottom}" class="grid"/>`).join("\n  ")}
  ${ticksY.map((tick) => `<line x1="${margin.left}" y1="${y(tick).toFixed(1)}" x2="${width - margin.right}" y2="${y(tick).toFixed(1)}" class="grid"/>`).join("\n  ")}
  <line x1="${margin.left}" y1="${height - margin.bottom}" x2="${width - margin.right}" y2="${height - margin.bottom}" class="axis"/>
  <line x1="${margin.left}" y1="${margin.top}" x2="${margin.left}" y2="${height - margin.bottom}" class="axis"/>
  ${ticksX.map((tick) => `<text x="${x(tick).toFixed(1)}" y="${height - margin.bottom + 42}" text-anchor="middle" class="tick">${tick}</text>`).join("\n  ")}
  ${ticksY.map((tick) => `<text x="${margin.left - 24}" y="${(y(tick) + 7).toFixed(1)}" text-anchor="end" class="tick">${tick.toFixed(2)}</text>`).join("\n  ")}

  <text x="${width / 2}" y="${height - 88}" text-anchor="middle" class="axis-label">Total rendered frames for 35 cases  ↓ denser</text>
  <text x="38" y="${height / 2 + 42}" transform="rotate(-90 38 ${height / 2 + 42})" text-anchor="middle" class="axis-label">Eligible F1  ↑ utility retained</text>

  <line x1="${margin.left}" y1="${y(text.eligible_avg_f1).toFixed(1)}" x2="${width - margin.right}" y2="${y(text.eligible_avg_f1).toFixed(1)}" class="ceiling"/>
  <text x="${width - margin.right - 10}" y="${(y(text.eligible_avg_f1) - 16).toFixed(1)}" text-anchor="end" class="ceiling-text">text ceiling ${text.eligible_avg_f1.toFixed(4)}</text>

  <path d="${frontierLine}" fill="none" stroke="url(#frontier)" stroke-width="8" stroke-linecap="round"/>
  <path d="${frontierLine}" fill="none" stroke="#ffffff" stroke-opacity="0.55" stroke-width="2.5" stroke-linecap="round"/>
  ${point(z18, "#7c3aed", "zpix18 · 18px · t0.49", 28, -112)}
  ${point(z24, "#0f4bd8", "zpix24 · 24px · binary", -350, -118)}

  <g transform="translate(1015 574)">
    <rect width="392" height="124" rx="22" fill="#0b163f" opacity="0.96"/>
    <text x="26" y="42" class="card-title">Best visual: zpix24</text>
    <text x="26" y="78" class="card-line">0.8753 F1 · 111 frames</text>
    <text x="26" y="112" class="card-line">7.7k chars/frame</text>
  </g>
  <text x="90" y="858" class="small">Eligible cases: ${text.eligible_cases}/${text.cases} where text baseline F1 ≥ ${results.baseline_threshold}. Results: reports/usf1-v0.1-longbench.md.</text>
</svg>
`;
  await fs.mkdir(path.dirname(outPath), { recursive: true });
  await Bun.write(outPath, svg);
  console.log(JSON.stringify({ outPath, width, height }, null, 2));
}

await main();
