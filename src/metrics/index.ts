import type { CoverageDetail, Finding, MetricEvaluation, Metrics, NormalizedHarness } from "../model.js";
import { getValidationProfile } from "../validation/profiles.js";

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
): Metrics {
  return {
    tokens: { count: estimateTokens(harnesses), tokenizer: "heuristic/4-chars" },
    cost: notEvaluated(),
    coverage: measureCoverage(harnesses, profileId),
    alignment: notEvaluated(),
    redundancy: measureRedundancy(harnesses),
    conflicts: findings.filter((item) => item.ruleId === "HL031").length,
  };
}
