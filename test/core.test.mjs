// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { discoverHarnessCandidates, scanRepository } from "../dist/index.js";

async function fixture(run) {
  const root = await mkdtemp(path.join(os.tmpdir(), "harness-lens-core-"));
  try {
    await run(root);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

test("discovers known files and assigns nested scope", () => fixture(async (root) => {
  await writeFile(path.join(root, "AGENTS.md"), "# Purpose\n- Always run tests\n");
  await mkdir(path.join(root, "packages", "api", ".cursor", "rules"), { recursive: true });
  await writeFile(path.join(root, "packages", "api", ".cursor", "rules", "local.md"), "- Must validate input\n");

  const candidates = await discoverHarnessCandidates(root);
  assert.equal(candidates.length, 2);
  assert.equal(candidates[1]?.kind, "cursor");
  assert.equal(candidates[1]?.scope, path.join(root, "packages", "api"));
}));

test("produces evidence and profile-bound metrics", () => fixture(async (root) => {
  await writeFile(path.join(root, "AGENTS.md"), [
    "# Purpose and scope",
    "- Always run tests",
    "- Never run tests",
    "- Use npm run build",
    "- Keep security constraints",
  ].join("\n"));

  const report = await scanRepository(root, {
    profile: "coding-agent/v1",
    now: () => new Date("2026-01-01T00:00:00.000Z"),
  });
  assert.equal(report.schemaVersion, "harness-lens/report/v1");
  assert.equal(report.metrics.coverage.status, "evaluated");
  assert.equal(report.metrics.coverage.reference, "coding-agent/v1");
  assert.ok(report.findings.some((finding) => finding.ruleId === "HL031"));
}));

test("does not invent coverage or alignment without references", () => fixture(async (root) => {
  await writeFile(path.join(root, "CLAUDE.md"), "# Instructions\n- Run tests\n");
  const report = await scanRepository(root);
  assert.equal(report.metrics.coverage.status, "not-evaluated");
  assert.equal(report.metrics.alignment.status, "not-evaluated");
  assert.equal(report.metrics.cost.status, "not-evaluated");
}));

test("reports exact duplicate lines with both source locations and normalization", () => fixture(async (root) => {
  await mkdir(path.join(root, "nested"), { recursive: true });
  await writeFile(path.join(root, "AGENTS.md"), "Adoption, rejection, assumptions, and source links.\n");
  await writeFile(path.join(root, "nested", "AGENTS.md"), "- adoption,   rejection, assumptions, and source links.\n");

  const report = await scanRepository(root);
  const duplicate = report.findings.find((finding) => finding.ruleId === "HL032");
  assert.ok(duplicate);
  assert.equal(duplicate.file, path.join(root, "nested", "AGENTS.md"));
  assert.deepEqual(duplicate.related, [{ file: path.join(root, "AGENTS.md"), line: 1 }]);
  assert.match(duplicate.evidence, /assumption:/);
  assert.equal(report.metrics.duplicates.lines, 1);
}));

test("evaluates source budgets and repeated-invocation input cost when configured", () => fixture(async (root) => {
  await writeFile(path.join(root, "AGENTS.md"), "Run the complete test suite.\n");
  const report = await scanRepository(root, {
    evaluation: {
      invocations: 10,
      inputCostPerMillionTokens: 2,
      costReference: "test-model/input-2026",
      maxSourceBytes: 1,
      maxSourceTokens: 1,
    },
  });

  assert.equal(report.metrics.cost.status, "evaluated");
  assert.equal(report.metrics.cost.reference, "test-model/input-2026");
  assert.equal(report.metrics.cost.details.inputTokensPerInvocation, report.metrics.tokens.count);
  assert.equal(report.metrics.cost.details.invocations, 10);
  assert.ok(report.metrics.cost.details.inputCostTotal > report.metrics.cost.details.inputCostPerInvocation);
  assert.ok(report.findings.some((finding) => finding.ruleId === "HL050"));
  assert.ok(report.findings.some((finding) => finding.ruleId === "HL051"));
}));
