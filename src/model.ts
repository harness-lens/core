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

export interface Metrics {
  tokens: {
    count: number;
    tokenizer: "heuristic/4-chars";
  };
  cost: MetricEvaluation;
  coverage: MetricEvaluation<CoverageDetail[]>;
  alignment: MetricEvaluation;
  redundancy: number;
  conflicts: number;
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
