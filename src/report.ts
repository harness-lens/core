import path from "node:path";
import { discoverHarnessCandidates } from "./discovery/candidates.js";
import { DEFAULT_MAX_FILE_BYTES, loadHarnessFile } from "./discovery/load.js";
import { calculateMetrics } from "./metrics/index.js";
import type { Finding, HarnessFile, HarnessReport, ScanOptions } from "./model.js";
import { parseHarness } from "./parser/markdown.js";
import { validateHarnesses } from "./validation/rules.js";

export async function scanRepository(repository: string, options: ScanOptions = {}): Promise<HarnessReport> {
  const root = path.resolve(repository);
  const candidates = await discoverHarnessCandidates(root);
  const files: HarnessFile[] = [];
  const loadFindings: Finding[] = [];

  for (const candidate of candidates) {
    try {
      files.push(await loadHarnessFile(candidate, options.maxFileBytes ?? DEFAULT_MAX_FILE_BYTES));
    } catch (error) {
      loadFindings.push({
        severity: "fail",
        ruleId: "HL006",
        message: "Harness file could not be loaded safely",
        file: candidate.path,
        line: null,
        evidence: error instanceof Error ? error.message : String(error),
      });
    }
  }

  const normalized = files.map(parseHarness);
  const findings = [...validateHarnesses(normalized, root), ...loadFindings];

  return {
    schemaVersion: "harness-lens/report/v1",
    repository: root,
    generatedAt: (options.now?.() ?? new Date()).toISOString(),
    files: files.map(({ path: filePath, kind, scope, bytes }) => ({ path: filePath, kind, scope, bytes })),
    findings,
    metrics: calculateMetrics(normalized, findings, options.profile),
  };
}
