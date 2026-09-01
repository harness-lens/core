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
