#!/usr/bin/env bun

import { $ } from "bun";
import * as path from "node:path";

const root = path.resolve(import.meta.dir, "..");
const profile = Bun.env.PREVIEW_PROFILE === "debug" ? "debug" : "release";
const bin = path.join(root, `target/${profile}/cjk-visual-context-bench`);
const defaultText = "fixtures/archives/omp-snapcompact-cjk-window-029/eventized.txt";
const port = Number(Bun.env.PORT ?? "8787");

type RenderRequest = {
  fontSize?: number;
  threshold?: number;
  lineSpacing?: number;
  frameSize?: number;
  contentWidth?: number;
  contentHeight?: number;
  margin?: number;
  text?: string;
};

function clamp(value: unknown, fallback: number, min: number, max: number): number {
  const number = typeof value === "number" && Number.isFinite(value) ? value : fallback;
  return Math.min(max, Math.max(min, number));
}

function variantName(request: Required<RenderRequest>): string {
  const threshold = String(Math.round(request.threshold * 100)).padStart(3, "0");
  return [
    `zpix${request.fontSize}`,
    `t${threshold}`,
    `l${request.lineSpacing}`,
    `f${request.frameSize}`,
    `c${request.contentWidth}x${request.contentHeight}`,
  ].join("-");
}

async function ensureBinary() {
  if (profile === "release") {
    await $`cargo build --release`.cwd(root).quiet();
  } else {
    await $`cargo build`.cwd(root).quiet();
  }
}

function safeRelative(input: string): string {
  const normalized = path.normalize(input).replace(/^\/+/, "");
  if (normalized.startsWith("..") || path.isAbsolute(normalized)) {
    throw new Error("path must stay inside preview project");
  }
  return normalized;
}

async function renderArchive(input: RenderRequest) {
  const request: Required<RenderRequest> = {
    fontSize: clamp(input.fontSize, 18, 8, 32),
    threshold: clamp(input.threshold, 0.58, 0.1, 0.9),
    lineSpacing: clamp(input.lineSpacing, 2, 0, 12),
    frameSize: clamp(input.frameSize, 2000, 512, 2400),
    contentWidth: clamp(input.contentWidth, 2000, 256, 2400),
    contentHeight: clamp(input.contentHeight, 2000, 256, 2400),
    margin: clamp(input.margin, 8, 0, 200),
    text: safeRelative(input.text ?? defaultText),
  };
  const name = variantName(request);
  const outDir = path.join("runs/preview", name);
  await ensureBinary();
  await $`${bin} render-archive --text ${request.text} --out-dir ${outDir} --name ${name} --font fonts/zpix.ttf --font-size ${String(request.fontSize)} --threshold ${String(request.threshold)} --line-spacing ${String(request.lineSpacing)} --frame-size ${String(request.frameSize)} --content-width ${String(request.contentWidth)} --content-height ${String(request.contentHeight)} --margin ${String(request.margin)}`.cwd(root).quiet();
  const manifest = await Bun.file(path.join(root, outDir, "manifest.json")).json();
  return {
    request,
    name,
    manifest,
    frames: manifest.frames.map((frame: { path: string }) => `/${outDir}/${frame.path}?v=${Date.now()}`),
  };
}

async function staticFile(relativePath: string): Promise<Response> {
  const fullPath = path.join(root, safeRelative(relativePath));
  if (!fullPath.startsWith(root)) return new Response("not found", { status: 404 });
  const file = Bun.file(fullPath);
  if (!(await file.exists())) return new Response("not found", { status: 404 });
  return new Response(file);
}

if (process.argv.includes("--smoke")) {
  const result = await renderArchive({ fontSize: 18, threshold: 0.58, lineSpacing: 2 });
  console.log(JSON.stringify({ name: result.name, frameCount: result.manifest.frame_count }, null, 2));
  process.exit(0);
}

Bun.serve({
  port,
  async fetch(request) {
    const url = new URL(request.url);
    try {
      if (url.pathname === "/") {
        return staticFile("web/preview.html");
      }
      if (url.pathname === "/api/render" && request.method === "POST") {
        const body = (await request.json()) as RenderRequest;
        return Response.json(await renderArchive(body));
      }
      if (url.pathname.startsWith("/runs/") || url.pathname.startsWith("/fixtures/")) {
        return staticFile(url.pathname.slice(1));
      }
      return new Response("not found", { status: 404 });
    } catch (error) {
      return Response.json({ error: error instanceof Error ? error.message : String(error) }, { status: 500 });
    }
  },
});

console.log(`preview: http://127.0.0.1:${port}`);
