// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

import path from "node:path";
import type { HarnessKind } from "../model.js";

export function inferScope(root: string, relativePath: string, kind: HarnessKind): string {
  const segments = relativePath.split("/");
  let scopeSegments: string[];

  if (kind === "copilot") {
    scopeSegments = segments.slice(0, segments.lastIndexOf(".github"));
  } else if (kind === "cursor") {
    scopeSegments = segments.slice(0, segments.lastIndexOf(".cursor"));
  } else {
    scopeSegments = segments.slice(0, -1);
  }

  return path.resolve(root, ...scopeSegments);
}
