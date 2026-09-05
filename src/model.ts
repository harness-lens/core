// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

export type HarnessKind = "agents" | "claude" | "gemini" | "copilot" | "cursor";

export interface HarnessCandidate {
  path: string;
  kind: HarnessKind;
  scope: string;
}

export interface HarnessFile extends HarnessCandidate {
  content: string;
  bytes: number;
}

export interface HarnessFileSummary extends HarnessCandidate {
  bytes: number;
}

export interface Directive {
  text: string;
  line: number;
}

export interface Heading {
  depth: number;
  text: string;
  line: number;
}

export interface NormalizedHarness {
  file: HarnessFile;
  text: string;
  headings: Heading[];
  directives: Directive[];
}

export type Severity = "pass" | "warn" | "fail";

export interface Finding {
  severity: Severity;
  ruleId: string;
  message: string;
  file: string;
  line: number | null;
  evidence: string | null;
  related?: FindingLocation[];
}

export interface FindingLocation {
  file: string;
  line: number | null;
}

export interface MetricEvaluation<TDetails = unknown> {
  status: "evaluated" | "not-evaluated";
  score: number | null;
  reference: string | null;
  details: TDetails | null;
}

export interface CoverageDetail {
  id: string;
  label: string;
  status: "present" | "missing";
}

export interface SourceMetrics {
  file: string;
  bytes: number;
  tokens: number;
  lines: number;
  paragraphs: number;
  tooLarge: boolean;
  overElaborated: boolean;
  costPerInvocation: number | null;
  costTotal: number | null;
}

export interface CostDetails {
  inputTokensPerInvocation: number;
  invocations: number;
  inputCostPerInvocation: number;
  inputCostTotal: number;
  currency: string;
}

export interface Metrics {
  tokens: {
    count: number;
    tokenizer: "heuristic/4-chars";
  };
  cost: MetricEvaluation<CostDetails>;
  coverage: MetricEvaluation<CoverageDetail[]>;
  alignment: MetricEvaluation;
  redundancy: number;
  conflicts: number;
  duplicates: {
    lines: number;
    paragraphs: number;
  };
  sources: SourceMetrics[];
  budgets: {
    maxSourceBytes: number;
    maxSourceTokens: number;
    tooLarge: number;
    overElaborated: number;
  };
}

export interface HarnessReport {
  schemaVersion: "harness-lens/report/v1";
  repository: string;
  generatedAt: string;
  files: HarnessFileSummary[];
  findings: Finding[];
  metrics: Metrics;
}

export interface ScanOptions {
  profile?: string;
  maxFileBytes?: number;
  now?: () => Date;
  evaluation?: {
    invocations?: number;
    inputCostPerMillionTokens?: number;
    costReference?: string;
    currency?: string;
    maxSourceBytes?: number;
    maxSourceTokens?: number;
  };
}

export interface ReportComparison {
  from: string;
  to: string;
  fileDelta: number;
  tokenDelta: number;
  findingDelta: number;
  conflictDelta: number;
  coverageDelta: number | null;
}
