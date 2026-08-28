import { readdir } from "node:fs/promises";
import path from "node:path";
import type { HarnessCandidate, HarnessKind } from "../model.js";
import { inferScope } from "./scope.js";

const SKIPPED_DIRECTORIES = new Set([
  ".git",
  ".hg",
  ".svn",
  "coverage",
  "dist",
  "node_modules",
  "target",
  "vendor",
]);

const DIRECT_NAMES = new Map<string, HarnessKind>([
  ["AGENTS.md", "agents"],
  ["CLAUDE.md", "claude"],
  ["GEMINI.md", "gemini"],
]);

export function classifyHarnessPath(relativePath: string): HarnessKind | null {
  const normalized = relativePath.replaceAll("\\", "/");
  const segments = normalized.split("/");
  const direct = DIRECT_NAMES.get(segments.at(-1) ?? "");
  if (direct) return direct;

  if (normalized.endsWith(".github/copilot-instructions.md")) return "copilot";

  const cursorIndex = segments.lastIndexOf(".cursor");
  if (cursorIndex >= 0 && segments[cursorIndex + 1] === "rules" && segments.length > cursorIndex + 2) {
    return "cursor";
  }

  return null;
}

export async function discoverHarnessCandidates(root: string): Promise<HarnessCandidate[]> {
  const absoluteRoot = path.resolve(root);
  const candidates: HarnessCandidate[] = [];

  async function walk(directory: string): Promise<void> {
    const entries = await readdir(directory, { withFileTypes: true });
    await Promise.all(entries.map(async (entry) => {
      if (entry.isSymbolicLink()) return;
      const absolute = path.join(directory, entry.name);

      if (entry.isDirectory()) {
        if (!SKIPPED_DIRECTORIES.has(entry.name)) await walk(absolute);
        return;
      }

      if (!entry.isFile()) return;
      const relative = path.relative(absoluteRoot, absolute).replaceAll(path.sep, "/");
      const kind = classifyHarnessPath(relative);
      if (!kind) return;

      candidates.push({
        path: absolute,
        kind,
        scope: inferScope(absoluteRoot, relative, kind),
      });
    }));
  }

  await walk(absoluteRoot);
  return candidates.sort((left, right) => left.path.localeCompare(right.path));
}
