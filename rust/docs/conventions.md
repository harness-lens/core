<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Copyright © 2026 Cristian Camargo Filho -->

# Checked harness conventions

Harness Lens turns documented provider behavior into conservative diagnostics.
Guidance that cannot be verified mechanically remains documentation, not a rule.

## Instruction files

- `HL040` reports empty harness sources.
- `HL041` reports `CLAUDE.md` and `CLAUDE.local.md` files over 200 lines. Anthropic
  recommends keeping each file under 200 lines because longer files consume more
  context and reduce adherence.
- `HL042` reports an individual `AGENTS.md` or `AGENTS.override.md` over 32 KiB.
  Codex stops adding project instructions when the combined chain reaches its
  default 32 KiB `project_doc_max_bytes` limit.
- `HL043` reports vague phrases such as “properly”, “best practices”, and “as
  needed”. This is heuristic: both OpenAI and Anthropic recommend specific,
  concrete, verifiable instructions.
- `HL044` reports an unclosed Markdown code fence because following instructions
  may be interpreted as code or skipped by text analyzers.
- `HL045` reports a GitHub path-specific `*.instructions.md` file without valid
  YAML frontmatter containing a non-empty `applyTo` string.

## Agent Skills

`HL100` through `HL105` validate the portable Agent Skills specification:

- YAML frontmatter is present and parseable.
- `name` and `description` are present strings within their length limits.
- `name` uses lowercase ASCII letters, digits, and single hyphens, and matches
  the skill directory name.
- the Markdown instruction body is non-empty.
- the main `SKILL.md` stays at or below the recommended 500 lines.

Harness Lens discovers `SKILL.md` entry points in `.agents/skills/`,
`.claude/skills/`, and other configured skill roots. Supporting scripts and
references remain available to the agent, but are not instruction documents and
are not linted as prose.

## Structured Codex assets

- `HL110` reports invalid TOML in `.codex/config.toml` and
  `.codex/agents/*.toml`.
- `HL111` through `HL113` require each Codex custom agent to define non-empty
  string values for `name`, `description`, and `developer_instructions`.
- `HL114` reports an empty `.codex/rules/*.rules` file or one without a
  `prefix_rule(...)` declaration.
- `HL115` reports a literal rule decision other than `allow`, `prompt`, or
  `forbidden`.

Harness Lens also discovers provider rule and agent Markdown under
`.claude/rules/`, `.claude/agents/`, `.cursor/rules/`, `.github/instructions/`,
and `.github/agents/`. `.agents/rules/` is recognized as an ecosystem extension,
not as an OpenAI-defined directory.

## Markdown agent profiles

- `HL120` reports invalid or missing YAML frontmatter in Claude and GitHub
  custom-agent profiles.
- `HL121` reports a missing or empty `description`.
- `HL122` validates Claude's required lowercase, hyphenated agent `name`.
- `HL123` reports an empty Markdown instruction body.
- `HL124` reports a GitHub agent prompt over its 30,000-character limit.

## Primary sources

- [OpenAI: Custom instructions with AGENTS.md](https://learn.chatgpt.com/docs/agent-configuration/agents-md)
- [OpenAI: Build skills](https://learn.chatgpt.com/docs/build-skills)
- [OpenAI: Config basics](https://learn.chatgpt.com/docs/config-file/config-basic)
- [OpenAI: Custom subagents](https://learn.chatgpt.com/docs/agent-configuration/subagents)
- [OpenAI: Rules](https://learn.chatgpt.com/docs/agent-configuration/rules)
- [Anthropic: How Claude remembers your project](https://code.claude.com/docs/en/memory)
- [Anthropic: Extend Claude with skills](https://code.claude.com/docs/en/slash-commands)
- [Anthropic: Create custom subagents](https://code.claude.com/docs/en/sub-agents)
- [GitHub: Adding repository custom instructions](https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/add-custom-instructions/add-repository-instructions)
- [GitHub: Custom agents configuration](https://docs.github.com/en/copilot/reference/custom-agents-configuration)
- [Agent Skills specification](https://agentskills.io/specification)
