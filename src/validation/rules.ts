// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

import type { Finding, NormalizedHarness } from "../model.js";

const AMBIGUOUS_PATTERN = /\b(?:maybe|usually|if needed|as appropriate|where possible)\b|\betc\./i;
const NEGATIVE_PATTERN = /\b(?:must not|never|do not|don't)\b/i;
const POSITIVE_PATTERN = /\b(?:must|always)\b/i;

function finding(
  severity: Finding["severity"],
  ruleId: string,
  message: string,
  file: string,
  line: number | null = null,
  evidence: string | null = null,
): Finding {
  return { severity, ruleId, message, file, line, evidence };
}

function directiveKey(text: string): string {
  return text
    .toLowerCase()
    .replace(/\b(?:must not|do not|don't|never|must|always)\b/g, "")
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}

export function validateHarnesses(harnesses: NormalizedHarness[], repository: string): Finding[] {
  if (harnesses.length === 0) {
    return [finding("fail", "HL001", "No recognized harness file found", repository)];
  }

  const findings: Finding[] = [
    finding("pass", "HL001", "Harness file found", harnesses[0]?.file.path ?? repository),
  ];

  for (const harness of harnesses) {
    findings.push(finding("pass", "HL006", "Valid UTF-8", harness.file.path));

    const testDirective = harness.directives.find((directive) => /\b(?:test|pytest|cargo test|npm test)\b/i.test(directive.text));
    findings.push(testDirective
      ? finding("pass", "HL014", "Testing instructions present", harness.file.path, testDirective.line, testDirective.text)
      : finding("warn", "HL014", "Testing instructions missing", harness.file.path));

    for (const directive of harness.directives) {
      if (AMBIGUOUS_PATTERN.test(directive.text)) {
        findings.push(finding("warn", "HL021", "Ambiguous instruction", harness.file.path, directive.line, directive.text));
      }
    }
  }

  const directives = harnesses.flatMap((harness) => harness.directives.map((directive) => ({
    ...directive,
    file: harness.file.path,
    scope: harness.file.scope,
  })));
  const groups = new Map<string, typeof directives>();
  for (const directive of directives) {
    const key = directiveKey(directive.text);
    if (!key) continue;
    const group = groups.get(key) ?? [];
    group.push(directive);
    groups.set(key, group);
  }

  for (const group of groups.values()) {
    const positive = group.find((directive) => POSITIVE_PATTERN.test(directive.text) && !NEGATIVE_PATTERN.test(directive.text));
    const negative = group.find((directive) => NEGATIVE_PATTERN.test(directive.text));
    if (!positive || !negative) continue;
    findings.push(finding(
      "fail",
      "HL031",
      "Conflicting instructions",
      negative.file,
      negative.line,
      `${positive.text} <> ${negative.text}`,
    ));
  }

  return findings;
}
