// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

import { readFile, stat } from "node:fs/promises";
import type { HarnessCandidate, HarnessFile } from "../model.js";

export const DEFAULT_MAX_FILE_BYTES = 1024 * 1024;

export async function loadHarnessFile(
  candidate: HarnessCandidate,
  maxFileBytes = DEFAULT_MAX_FILE_BYTES,
): Promise<HarnessFile> {
  const metadata = await stat(candidate.path);
  if (!metadata.isFile()) throw new Error("Harness candidate is not a regular file");
  if (metadata.size > maxFileBytes) {
    throw new Error(`Harness file exceeds ${maxFileBytes} bytes`);
  }

  const bytes = await readFile(candidate.path);
  const content = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  return { ...candidate, content, bytes: bytes.byteLength };
}
