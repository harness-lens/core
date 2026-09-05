> SPDX-License-Identifier: MPL-2.0
> Copyright © 2026 Cristian Camargo Filho

# Harness Lens rule index

Rule IDs are stable report codes. The primary location is the `path`/`line`
pair on a finding; relationship-based rules may also emit `related` locations.
The Rust implementation is the reference implementation. The TypeScript
implementation is kept for npm compatibility.

The `HL040`–`HL045` range is already reserved by the newer Rust convention
checks. The source-size and token-budget rules therefore use `HL050`–`HL052`.

| ID | Severity | What it reports | Assumption or reference | Implementation |
| --- | --- | --- | --- | --- |
| `HL001` | pass/warn/fail | Harness source presence | A scan with no recognized source is incomplete | [Rust inventory](../rust/src/engine.rs), [TS validation](../src/validation/rules.ts) |
| `HL006` | pass/fail | Safe UTF-8 source loading | Files must be regular files and valid UTF-8 | [TS report](../src/report.ts) |
| `HL010` | warn | Adjacent repeated word | Case-insensitive word comparison; fenced code is ignored | [Rust text analysis](../rust/src/text_analysis.rs) |
| `HL014` | pass/warn | Testing instructions present or missing | A test-related directive is sufficient | [TS validation](../src/validation/rules.ts) |
| `HL020` | warn | Opposite strong instructions in overlapping scopes | Only exact normalized `always`/`never` and `must`/`must not` pairs conflict | [Rust text analysis](../rust/src/text_analysis.rs) |
| `HL021` | warn | Ambiguous instruction language | Conservative phrase list: `maybe`, `usually`, `if needed`, `etc.` and similar | [TS validation](../src/validation/rules.ts) |
| `HL030` | warn | Substantially redundant instruction intent | Same-polarity directives with at least 80% target coverage and 70% Jaccard similarity | [Rust latest text analysis](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/text_analysis.rs) |
| `HL031` | fail | Conflicting instructions | TypeScript compatibility rule; Rust uses `HL020` for this condition | [TS validation](../src/validation/rules.ts) |
| `HL032` | warn | Exact duplicate line or multi-line paragraph | Lowercase, trim/collapse whitespace, remove Markdown markers, ignore fenced code; later finding points to earlier `related` location | [Rust rule](../rust/src/exact_duplicates.rs), [TS rule](../src/validation/exact-duplicates.ts) |
| `HL040` | warn | Empty instruction file | Empty source provides no project guidance | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL041` | warn | `CLAUDE.md` over 200 lines | Anthropic’s documented 200-line target | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL042` | warn | `AGENTS.md` over 32 KiB | Codex `project_doc_max_bytes` default | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL043` | info | Vague phrases such as “properly” or “best practices” | Heuristic; specific, verifiable instructions are preferred | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL044` | warn | Unclosed Markdown code fence | Text after an opening fence may be interpreted as code | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL045` | error | Path-specific Copilot file without valid `applyTo` frontmatter | GitHub path-specific instructions require a non-empty `applyTo` glob | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL050` | warn | Source exceeds the configured byte budget | Soft threshold; configure `evaluation.max_source_bytes` | [Rust evaluation](../rust/src/evaluation.rs), [TS report](../src/report.ts) |
| `HL051` | warn | Source exceeds the configured estimated-token budget | Soft heuristic; estimated tokens are `ceil(character count / 4)` | [Rust evaluation](../rust/src/evaluation.rs), [TS report](../src/report.ts) |
| `HL052` | error | Invalid input token price | Cost is not calculated for a non-finite or negative price | [Rust evaluation](../rust/src/evaluation.rs), [TS report](../src/report.ts) |
| `HL100` | error | Agent Skill frontmatter missing or invalid | Portable Agent Skills frontmatter must parse as YAML | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL101` | error | Skill name missing or invalid | Lowercase ASCII name, 1–64 characters, single hyphens | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL102` | error | Skill name does not match its directory | Required by the Agent Skills specification | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL103` | error | Skill description missing, empty, or over 1,024 characters | Description must explain purpose and activation context | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL104` | error | Skill instruction body empty | `SKILL.md` must contain instructions after frontmatter | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL105` | warn | Skill main file over 500 lines | Move longer detail into referenced supporting files | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL110` | error | Invalid Codex TOML asset | `.codex/config.toml` and custom agents must parse as TOML | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL111` | error | Codex custom-agent `name` missing or empty | Required non-empty string | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL112` | error | Codex custom-agent `description` missing or empty | Required non-empty string | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL113` | error | Codex custom-agent `developer_instructions` missing or empty | Required non-empty string | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL114` | error | Codex `.rules` file empty or missing `prefix_rule(...)` | Rules must declare command policy | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL115` | error | Unknown Codex rule decision | Decision must be `allow`, `prompt`, or `forbidden` | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL120` | error | Claude/GitHub agent profile frontmatter invalid | Agent profile requires valid YAML frontmatter | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL121` | error | Agent profile description missing or empty | Description identifies purpose and delegation scope | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL122` | error | Claude agent name invalid | Lowercase letters and hyphens | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL123` | error | Agent instruction body empty | Markdown body defines the agent behavior | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |
| `HL124` | error | GitHub custom-agent prompt over 30,000 characters | GitHub documented prompt limit | [Rust latest conventions](https://github.com/harness-lens/core/blob/855598e5ef83b464aa799f8515d3a9170ac88abc/rust/src/conventions.rs) |

## Metrics are not rules

The evaluation plugin also emits measurements without creating a finding:

- `harness.source.bytes`, `.estimated_tokens`, `.lines`, and `.paragraphs`
  identify per-source size and complexity.
- `harness.total_estimated_tokens` is the estimated input cost basis for one
  invocation.
- `harness.input_cost_per_invocation` and `harness.input_cost_total` are
  emitted only when the caller supplies `input_cost_per_million_tokens`.

The cost assumption is input injection only:
`total = estimated tokens × input price × invocation count`. The tokenizer is a
heuristic, not a provider-specific billing tokenizer.
