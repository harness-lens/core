import type { FindingLocation, NormalizedHarness } from "../model.js";

export const EXACT_DUPLICATE_NORMALIZATION =
  "lowercase; trim and collapse whitespace; remove Markdown heading/list markers and emphasis backticks; ignore fenced code";

export type ExactDuplicateKind = "line" | "paragraph";

export interface ExactDuplicate {
  kind: ExactDuplicateKind;
  file: string;
  line: number;
  previous: FindingLocation;
  evidence: string;
}

interface TextUnit {
  kind: ExactDuplicateKind;
  normalized: string;
  file: string;
  scope: string;
  line: number;
}

export function findExactDuplicates(harnesses: NormalizedHarness[]): ExactDuplicate[] {
  const groups = new Map<string, TextUnit[]>();
  for (const harness of harnesses) {
    for (const unit of extractUnits(harness)) {
      if (unit.normalized.length < 8) continue;
      const key = `${unit.kind}\u0000${unit.normalized}`;
      const group = groups.get(key) ?? [];
      group.push(unit);
      groups.set(key, group);
    }
  }

  const duplicates: ExactDuplicate[] = [];
  for (const units of groups.values()) {
    units.sort((left, right) => left.file.localeCompare(right.file) || left.line - right.line);
    for (let index = 1; index < units.length; index += 1) {
      const current = units[index];
      if (!current) continue;
      let previous: TextUnit | undefined;
      for (let previousIndex = index - 1; previousIndex >= 0; previousIndex -= 1) {
        const candidate = units[previousIndex];
        if (candidate && scopesOverlap(current.scope, candidate.scope)) {
          previous = candidate;
          break;
        }
      }
      if (!previous) continue;
      duplicates.push({
        kind: current.kind,
        file: current.file,
        line: current.line,
        previous: { file: previous.file, line: previous.line },
        evidence: `normalized ${current.kind} matches exactly; assumption: ${EXACT_DUPLICATE_NORMALIZATION}`,
      });
    }
  }
  return duplicates.sort((left, right) => left.file.localeCompare(right.file) || left.line - right.line);
}

function extractUnits(harness: NormalizedHarness): TextUnit[] {
  const units: TextUnit[] = [];
  const paragraph: Array<{ line: number; text: string }> = [];
  let inFence = false;
  const flushParagraph = (): void => {
    if (paragraph.length > 1) {
      const normalized = normalizeExact(paragraph.map((part) => part.text).join(" "));
      if (normalized) {
        units.push({
          kind: "paragraph",
          normalized,
          file: harness.file.path,
          scope: harness.file.scope,
          line: paragraph[0]?.line ?? 1,
        });
      }
    }
    paragraph.length = 0;
  };

  for (const [index, raw] of harness.text.split("\n").entries()) {
    const line = index + 1;
    if (isFence(raw)) {
      flushParagraph();
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;
    if (!raw.trim()) {
      flushParagraph();
      continue;
    }
    const normalized = normalizeExact(raw);
    if (!normalized) continue;
    units.push({ kind: "line", normalized, file: harness.file.path, scope: harness.file.scope, line });
    paragraph.push({ line, text: raw });
  }
  flushParagraph();
  return units;
}

function normalizeExact(value: string): string {
  let text = value.trim();
  while (text.startsWith("#")) text = text.slice(1).trimStart();
  const list = /^(?:[-*+]\s+|\d+[.)]\s+)/.exec(text);
  if (list) text = text.slice(list[0].length).trimStart();
  return text.toLowerCase().replaceAll("`", "").replaceAll("*", "").split(/\s+/).join(" ");
}

function isFence(line: string): boolean {
  const trimmed = line.trimStart();
  return trimmed.startsWith("```") || trimmed.startsWith("~~~");
}

function scopesOverlap(left: string, right: string): boolean {
  const normalizedLeft = left.replaceAll("\\", "/").replace(/\/+$/, "");
  const normalizedRight = right.replaceAll("\\", "/").replace(/\/+$/, "");
  return normalizedLeft === normalizedRight
    || normalizedLeft.startsWith(`${normalizedRight}/`)
    || normalizedRight.startsWith(`${normalizedLeft}/`);
}
