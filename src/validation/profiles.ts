export interface ProfileRequirement {
  id: string;
  label: string;
  patterns: RegExp[];
}

export interface ValidationProfile {
  id: string;
  requirements: ProfileRequirement[];
}

export const CODING_AGENT_V1: ValidationProfile = {
  id: "coding-agent/v1",
  requirements: [
    { id: "purpose", label: "Purpose and scope", patterns: [/\bpurpose\b/i, /\bscope\b/i, /\bobjective\b/i] },
    { id: "build", label: "Build commands", patterns: [/\bbuild\b/i, /npm run build/i, /cargo build/i] },
    { id: "test", label: "Test commands", patterns: [/\btests?\b/i, /pytest/i, /cargo test/i] },
    { id: "editing", label: "File-editing policy", patterns: [/\bedit(?:ing)?\b/i, /apply_patch/i, /modify files/i] },
    { id: "security", label: "Security constraints", patterns: [/\bsecurity\b/i, /\bsecret\b/i, /credential/i] },
    { id: "dependencies", label: "Dependency policy", patterns: [/dependenc/i, /package manager/i, /lockfile/i] },
    { id: "output", label: "Output requirements", patterns: [/\boutput\b/i, /response format/i, /final answer/i] },
  ],
};

const PROFILES = new Map([[CODING_AGENT_V1.id, CODING_AGENT_V1]]);

export function getValidationProfile(id: string): ValidationProfile | null {
  return PROFILES.get(id) ?? null;
}
