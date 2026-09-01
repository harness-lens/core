// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

import type { Directive, HarnessFile, Heading, NormalizedHarness } from "../model.js";

const DIRECTIVE_PATTERN = /^\s*(?:[-*+]\s+|\d+[.)]\s+)(.+?)\s*$/;
const IMPERATIVE_PATTERN = /\b(?:always|never|must|must not|do not|don't|required|should)\b/i;

export function normalizeMarkdown(content: string): string {
  return content.replaceAll("\r\n", "\n").replaceAll("\r", "\n").trimEnd();
}

export function parseHarness(file: HarnessFile): NormalizedHarness {
  const text = normalizeMarkdown(file.content);
  const headings: Heading[] = [];
  const directives: Directive[] = [];

  for (const [index, line] of text.split("\n").entries()) {
    const heading = /^(#{1,6})\s+(.+?)\s*$/.exec(line);
    if (heading?.[1] && heading[2]) {
      headings.push({ depth: heading[1].length, text: heading[2], line: index + 1 });
    }

    const listed = DIRECTIVE_PATTERN.exec(line)?.[1];
    const directive = listed ?? (IMPERATIVE_PATTERN.test(line) ? line.trim() : null);
    if (directive) directives.push({ text: directive, line: index + 1 });
  }

  return { file, text, headings, directives };
}
