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
