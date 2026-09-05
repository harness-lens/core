// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

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
  const metrics = calculateMetrics(normalized, findings, options.profile, options.evaluation);
  for (const source of metrics.sources) {
    if (source.tooLarge) {
      findings.push({
        severity: "warn",
        ruleId: "HL050",
        message: `Harness source is too large: ${source.bytes} bytes exceeds ${metrics.budgets.maxSourceBytes}`,
        file: source.file,
        line: 1,
        evidence: "soft source-size budget; configure evaluation.maxSourceBytes",
      });
    }
    if (source.overElaborated) {
      findings.push({
        severity: "warn",
        ruleId: "HL051",
        message: `Harness source is over-elaborated: ${source.tokens} estimated tokens exceeds ${metrics.budgets.maxSourceTokens}`,
        file: source.file,
        line: 1,
        evidence: "soft token-budget heuristic; tokens are estimated as characters / 4",
      });
    }
  }
  const inputRate = options.evaluation?.inputCostPerMillionTokens;
  if (typeof inputRate === "number" && (!Number.isFinite(inputRate) || inputRate < 0)) {
    findings.push({
      severity: "fail",
      ruleId: "HL052",
      message: "Input token price must be finite and non-negative",
      file: root,
      line: null,
      evidence: "cost was not calculated because the caller supplied an invalid price",
    });
  }

  return {
    schemaVersion: "harness-lens/report/v1",
    repository: root,
    generatedAt: (options.now?.() ?? new Date()).toISOString(),
    files: files.map(({ path: filePath, kind, scope, bytes }) => ({ path: filePath, kind, scope, bytes })),
    findings,
    metrics,
  };
}
