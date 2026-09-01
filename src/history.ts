// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

import type { HarnessReport, ReportComparison } from "./model.js";

export interface SnapshotStore {
  save(report: HarnessReport): Promise<void>;
  load(reference: string): Promise<HarnessReport | null>;
}

export function compareReports(from: HarnessReport, to: HarnessReport): ReportComparison {
  const fromCoverage = from.metrics.coverage;
  const toCoverage = to.metrics.coverage;
  const coverageDelta = fromCoverage.status === "evaluated" && toCoverage.status === "evaluated"
    && fromCoverage.score !== null && toCoverage.score !== null
    ? toCoverage.score - fromCoverage.score
    : null;

  return {
    from: from.generatedAt,
    to: to.generatedAt,
    fileDelta: to.files.length - from.files.length,
    tokenDelta: to.metrics.tokens.count - from.metrics.tokens.count,
    findingDelta: to.findings.length - from.findings.length,
    conflictDelta: to.metrics.conflicts - from.metrics.conflicts,
    coverageDelta,
  };
}
