// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

import type { HarnessReport } from "./model.js";

export interface AiInterpretation {
  trendExplanation?: string;
  semanticGroups?: string[][];
  possibleConflicts?: string[];
  simplifications?: string[];
}

export interface AiInterpreter {
  interpret(report: Readonly<HarnessReport>): Promise<AiInterpretation>;
}
