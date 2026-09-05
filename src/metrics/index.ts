// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

import type {
  CoverageDetail,
  CostDetails,
  Finding,
  MetricEvaluation,
  Metrics,
  NormalizedHarness,
  SourceMetrics,
  ScanOptions,
} from "../model.js";
import { getValidationProfile } from "../validation/profiles.js";
import { findExactDuplicates } from "../validation/exact-duplicates.js";

function notEvaluated<T>(): MetricEvaluation<T> {
  return { status: "not-evaluated", score: null, reference: null, details: null };
}

export function estimateTokens(harnesses: NormalizedHarness[]): number {
  const characters = harnesses.reduce((total, harness) => total + harness.text.length, 0);
  return Math.ceil(characters / 4);
}

export function measureCoverage(
  harnesses: NormalizedHarness[],
  profileId?: string,
): MetricEvaluation<CoverageDetail[]> {
  if (!profileId) return notEvaluated<CoverageDetail[]>();
  const profile = getValidationProfile(profileId);
  if (!profile) return notEvaluated<CoverageDetail[]>();

  const corpus = harnesses.map((harness) => harness.text).join("\n");
  const details = profile.requirements.map((requirement): CoverageDetail => ({
    id: requirement.id,
    label: requirement.label,
    status: requirement.patterns.some((pattern) => pattern.test(corpus)) ? "present" : "missing",
  }));
  const present = details.filter((detail) => detail.status === "present").length;

  return {
    status: "evaluated",
    score: details.length === 0 ? 0 : present / details.length,
    reference: profile.id,
    details,
  };
}

export function measureRedundancy(harnesses: NormalizedHarness[]): number {
  const directives = harnesses.flatMap((harness) => harness.directives)
    .map((directive) => directive.text.toLowerCase().replace(/\s+/g, " ").trim())
    .filter(Boolean);
  if (directives.length === 0) return 0;
  return (directives.length - new Set(directives).size) / directives.length;
}

export function calculateMetrics(
  harnesses: NormalizedHarness[],
  findings: Finding[],
  profileId?: string,
  evaluation: ScanOptions["evaluation"] = {},
): Metrics {
  const invocations = Math.max(1, Math.floor(evaluation.invocations ?? 1));
  const maxSourceBytes = evaluation.maxSourceBytes ?? 32 * 1024;
  const maxSourceTokens = evaluation.maxSourceTokens ?? 8_000;
  const inputRate = evaluation.inputCostPerMillionTokens;
  const totalTokens = estimateTokens(harnesses);
  const exactDuplicates = findExactDuplicates(harnesses);
  const costEvaluated = typeof inputRate === "number" && Number.isFinite(inputRate) && inputRate >= 0;
  const inputCostPerInvocation = costEvaluated ? totalTokens * inputRate / 1_000_000 : null;
  const inputCostTotal = inputCostPerInvocation === null ? null : inputCostPerInvocation * invocations;
  const sourceMetrics: SourceMetrics[] = harnesses.map((harness) => {
    const tokens = Math.ceil(harness.text.length / 4);
    return {
      file: harness.file.path,
      bytes: harness.file.bytes,
      tokens,
      lines: harness.text ? harness.text.split("\n").length : 0,
      paragraphs: harness.text ? harness.text.split(/\n\s*\n/).filter(Boolean).length : 0,
      tooLarge: harness.file.bytes > maxSourceBytes,
      overElaborated: tokens > maxSourceTokens,
      costPerInvocation: costEvaluated ? tokens * inputRate / 1_000_000 : null,
      costTotal: costEvaluated ? tokens * inputRate / 1_000_000 * invocations : null,
    };
  });
  const costDetails: CostDetails | null = inputCostPerInvocation === null ? null : {
    inputTokensPerInvocation: totalTokens,
    invocations,
    inputCostPerInvocation,
    inputCostTotal: inputCostTotal ?? 0,
    currency: evaluation.currency ?? "USD",
  };

  return {
    tokens: { count: totalTokens, tokenizer: "heuristic/4-chars" },
    cost: costDetails === null
      ? notEvaluated<CostDetails>()
      : {
        status: "evaluated",
        score: null,
        reference: evaluation.costReference ?? "caller-supplied input rate",
        details: costDetails,
      },
    coverage: measureCoverage(harnesses, profileId),
    alignment: notEvaluated(),
    redundancy: measureRedundancy(harnesses),
    conflicts: findings.filter((item) => item.ruleId === "HL031").length,
    duplicates: {
      lines: exactDuplicates.filter((duplicate) => duplicate.kind === "line").length,
      paragraphs: exactDuplicates.filter((duplicate) => duplicate.kind === "paragraph").length,
    },
    sources: sourceMetrics,
    budgets: {
      maxSourceBytes,
      maxSourceTokens,
      tooLarge: sourceMetrics.filter((source) => source.tooLarge).length,
      overElaborated: sourceMetrics.filter((source) => source.overElaborated).length,
    },
  };
}
