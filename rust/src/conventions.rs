// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

use std::path::Path;

use serde_yaml::Value;

use crate::{
    Finding, HarnessSource, HarnessSourceKind, Metric, Plugin, PluginContext, PluginError,
    PluginMetadata, PluginOutput, Score, ScoreCategory, ScoreMethod, Severity, TextSpan,
};

const INSTRUCTION_PLUGIN_ID: &str = "harness-lens.instruction-conventions";
const SKILL_PLUGIN_ID: &str = "harness-lens.skill-conventions";
const CODEX_ASSET_PLUGIN_ID: &str = "harness-lens.codex-asset-conventions";
const CLAUDE_TARGET_LINES: usize = 200;
const CODEX_DEFAULT_BYTES: usize = 32 * 1024;
const SKILL_RECOMMENDED_LINES: usize = 500;

pub(crate) struct InstructionConventionsPlugin;

impl Plugin for InstructionConventionsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: INSTRUCTION_PLUGIN_ID.to_owned(),
            name: "Instruction-file conventions".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            capabilities: vec!["instructions.conventions".to_owned()],
            default_enabled: true,
        }
    }

    fn analyze(&self, context: &PluginContext<'_>) -> Result<PluginOutput, PluginError> {
        let mut findings = Vec::new();
        let mut checked = 0_usize;

        for source in context
            .sources
            .iter()
            .filter(|source| source.kind != HarnessSourceKind::Skills && is_markdown_source(source))
        {
            checked += 1;
            check_instruction_source(source, &mut findings);
        }

        Ok(convention_output(
            findings,
            checked,
            "harness.instruction_conventions_valid",
            "harness.instruction_convention_findings",
            INSTRUCTION_PLUGIN_ID,
            ScoreMethod::Heuristic,
            "provider limits use documented defaults; vague-language matching is conservative",
        ))
    }
}

pub(crate) struct CodexAssetConventionsPlugin;

impl Plugin for CodexAssetConventionsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: CODEX_ASSET_PLUGIN_ID.to_owned(),
            name: "Codex project asset conventions".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            capabilities: vec!["codex.assets.validation".to_owned()],
            default_enabled: true,
        }
    }

    fn analyze(&self, context: &PluginContext<'_>) -> Result<PluginOutput, PluginError> {
        let mut findings = Vec::new();
        let mut checked = 0_usize;

        for source in context
            .sources
            .iter()
            .filter(|source| is_codex_asset(source))
        {
            checked += 1;
            check_codex_asset(source, &mut findings);
        }

        Ok(convention_output(
            findings,
            checked,
            "harness.codex_assets_valid",
            "harness.codex_asset_convention_findings",
            CODEX_ASSET_PLUGIN_ID,
            ScoreMethod::Deterministic,
            "validation covers documented TOML requirements and conservative rule-file structure",
        ))
    }
}

pub(crate) struct SkillConventionsPlugin;

impl Plugin for SkillConventionsPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: SKILL_PLUGIN_ID.to_owned(),
            name: "Agent Skill conventions".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            capabilities: vec!["skills.validation".to_owned()],
            default_enabled: true,
        }
    }

    fn analyze(&self, context: &PluginContext<'_>) -> Result<PluginOutput, PluginError> {
        let mut findings = Vec::new();
        let mut checked = 0_usize;

        for source in context
            .sources
            .iter()
            .filter(|source| source.kind == HarnessSourceKind::Skills)
        {
            checked += 1;
            check_skill(source, &mut findings);
        }

        Ok(convention_output(
            findings,
            checked,
            "harness.skills_valid",
            "harness.skill_convention_findings",
            SKILL_PLUGIN_ID,
            ScoreMethod::Deterministic,
            "validation follows the portable Agent Skills frontmatter and naming specification",
        ))
    }
}

fn check_instruction_source(source: &HarnessSource, findings: &mut Vec<Finding>) {
    if source.content.trim().is_empty() {
        findings.push(source_finding(
            source,
            Severity::Warning,
            "HL040",
            "Instruction file is empty",
            (Some(1), Some(TextSpan { start: 0, end: 0 })),
            "empty instruction files are ignored or provide no project guidance",
            INSTRUCTION_PLUGIN_ID,
        ));
        return;
    }

    let name = file_name(source);
    let line_count = source.content.lines().count();
    if matches!(name, Some("CLAUDE.md" | "CLAUDE.local.md")) && line_count > CLAUDE_TARGET_LINES {
        findings.push(source_finding(
            source,
            Severity::Warning,
            "HL041",
            format!(
                "Claude instruction file has {line_count} lines; target at most {CLAUDE_TARGET_LINES}"
            ),
            (
                Some(CLAUDE_TARGET_LINES + 1),
                line_span(&source.content, CLAUDE_TARGET_LINES + 1),
            ),
            "Anthropic recommends keeping each CLAUDE.md under 200 lines",
            INSTRUCTION_PLUGIN_ID,
        ));
    }

    if matches!(name, Some("AGENTS.md" | "AGENTS.override.md"))
        && source.content.len() > CODEX_DEFAULT_BYTES
    {
        findings.push(source_finding(
            source,
            Severity::Warning,
            "HL042",
            format!(
                "Codex instruction file exceeds the default {CODEX_DEFAULT_BYTES}-byte combined budget"
            ),
            (Some(1), line_span(&source.content, 1)),
            "one file alone exceeds Codex project_doc_max_bytes default",
            INSTRUCTION_PLUGIN_ID,
        ));
    }

    check_vague_language(source, findings);

    if let Some((line, span)) = unclosed_fence(&source.content) {
        findings.push(source_finding(
            source,
            Severity::Warning,
            "HL044",
            "Markdown code fence is not closed",
            (Some(line), Some(span)),
            "content after the opening fence may be treated as code instead of instructions",
            INSTRUCTION_PLUGIN_ID,
        ));
    }

    if name.is_some_and(|name| name.ends_with(".instructions.md")) {
        match parse_frontmatter(&source.content) {
            Ok(frontmatter)
                if string_field(&frontmatter.value, "applyTo")
                    .is_some_and(|v| !v.trim().is_empty()) => {}
            _ => findings.push(source_finding(
                source,
                Severity::Error,
                "HL045",
                "Path-specific Copilot instructions require an applyTo glob",
                (Some(1), line_span(&source.content, 1)),
                "GitHub requires YAML frontmatter with a non-empty applyTo field",
                INSTRUCTION_PLUGIN_ID,
            )),
        }
    }

    if is_claude_agent(source) || is_github_agent(source) {
        check_markdown_agent(source, findings);
    }
}

fn check_markdown_agent(source: &HarnessSource, findings: &mut Vec<Finding>) {
    let frontmatter = match parse_frontmatter(&source.content) {
        Ok(frontmatter) => frontmatter,
        Err(error) => {
            findings.push(source_finding(
                source,
                Severity::Error,
                "HL120",
                "Agent profile requires valid YAML frontmatter",
                (Some(error.line), line_span(&source.content, error.line)),
                error.message,
                INSTRUCTION_PLUGIN_ID,
            ));
            return;
        }
    };

    if string_field(&frontmatter.value, "description")
        .is_none_or(|description| description.trim().is_empty())
    {
        findings.push(yaml_field_finding(
            source,
            Severity::Error,
            "HL121",
            "Agent profile requires a non-empty description",
            "description",
            "the provider uses description to identify the agent's purpose and delegation scope",
            INSTRUCTION_PLUGIN_ID,
        ));
    }

    if is_claude_agent(source)
        && string_field(&frontmatter.value, "name").is_none_or(|name| {
            name.is_empty()
                || name.starts_with('-')
                || name.ends_with('-')
                || !name
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
        })
    {
        findings.push(yaml_field_finding(
            source,
            Severity::Error,
            "HL122",
            "Claude agent requires a lowercase name using letters and hyphens",
            "name",
            "Anthropic requires name and description in Claude subagent frontmatter",
            INSTRUCTION_PLUGIN_ID,
        ));
    }

    if frontmatter.body.trim().is_empty() {
        findings.push(source_finding(
            source,
            Severity::Error,
            "HL123",
            "Agent instruction body is empty",
            (
                Some(frontmatter.body_line),
                Some(TextSpan {
                    start: frontmatter.body_start,
                    end: frontmatter.body_start,
                }),
            ),
            "the Markdown body defines the agent's behavior",
            INSTRUCTION_PLUGIN_ID,
        ));
    }

    if is_github_agent(source) && frontmatter.body.chars().count() > 30_000 {
        findings.push(source_finding(
            source,
            Severity::Error,
            "HL124",
            "GitHub custom-agent prompt exceeds 30,000 characters",
            (
                Some(frontmatter.body_line),
                Some(TextSpan {
                    start: frontmatter.body_start,
                    end: frontmatter.body_start,
                }),
            ),
            "GitHub limits the Markdown prompt below agent frontmatter to 30,000 characters",
            INSTRUCTION_PLUGIN_ID,
        ));
    }
}

fn check_vague_language(source: &HarnessSource, findings: &mut Vec<Finding>) {
    const PHRASES: [&str; 8] = [
        "properly",
        "appropriately",
        "best practices",
        "clean code",
        "as needed",
        "when necessary",
        "etc.",
        "and so on",
    ];
    let mut fence = None;
    for (line_number, line_start, line) in source_lines(&source.content) {
        if let Some(marker) = fence_marker(line) {
            if fence == Some(marker) {
                fence = None;
            } else if fence.is_none() {
                fence = Some(marker);
            }
            continue;
        }
        if fence.is_some() {
            continue;
        }

        let lowered = line.to_lowercase();
        let Some(phrase) = PHRASES
            .iter()
            .find(|phrase| contains_phrase(&lowered, phrase))
        else {
            continue;
        };
        findings.push(source_finding(
            source,
            Severity::Info,
            "HL043",
            format!("Instruction uses vague phrase “{phrase}”"),
            (Some(line_number), Some(full_line_span(line_start, line))),
            "specific, concrete, verifiable instructions are more reliably followed",
            INSTRUCTION_PLUGIN_ID,
        ));
    }
}

fn check_skill(source: &HarnessSource, findings: &mut Vec<Finding>) {
    let frontmatter = match parse_frontmatter(&source.content) {
        Ok(frontmatter) => frontmatter,
        Err(error) => {
            findings.push(source_finding(
                source,
                Severity::Error,
                "HL100",
                "SKILL.md requires valid YAML frontmatter",
                (Some(error.line), line_span(&source.content, error.line)),
                error.message,
                SKILL_PLUGIN_ID,
            ));
            return;
        }
    };

    let name = string_field(&frontmatter.value, "name");
    match name {
        Some(name) if valid_skill_name(name) => {}
        _ => findings.push(field_finding(
            source,
            "HL101",
            "Skill name is missing or invalid",
            "name",
            "name must be 1-64 lowercase ASCII letters, digits, or single hyphens",
        )),
    }
    if let (Some(name), Some(directory)) = (
        name.filter(|name| valid_skill_name(name)),
        source
            .path
            .parent()
            .and_then(Path::file_name)
            .and_then(|v| v.to_str()),
    ) {
        if name != directory {
            findings.push(field_finding(
                source,
                "HL102",
                format!("Skill name “{name}” does not match directory “{directory}”"),
                "name",
                "the Agent Skills specification requires the name to match its directory",
            ));
        }
    }

    match string_field(&frontmatter.value, "description") {
        Some(description)
            if !description.trim().is_empty() && description.chars().count() <= 1024 => {}
        _ => findings.push(field_finding(
            source,
            "HL103",
            "Skill description is missing, empty, or over 1024 characters",
            "description",
            "description must explain what the skill does and when to use it",
        )),
    }

    if frontmatter.body.trim().is_empty() {
        findings.push(source_finding(
            source,
            Severity::Error,
            "HL104",
            "Skill instruction body is empty",
            (
                Some(frontmatter.body_line),
                Some(TextSpan {
                    start: frontmatter.body_start,
                    end: frontmatter.body_start,
                }),
            ),
            "SKILL.md must contain Markdown instructions after its frontmatter",
            SKILL_PLUGIN_ID,
        ));
    }

    let line_count = source.content.lines().count();
    if line_count > SKILL_RECOMMENDED_LINES {
        findings.push(source_finding(
            source,
            Severity::Warning,
            "HL105",
            format!(
                "SKILL.md has {line_count} lines; keep the main file at or below {SKILL_RECOMMENDED_LINES}"
            ),
            (
                Some(SKILL_RECOMMENDED_LINES + 1),
                line_span(&source.content, SKILL_RECOMMENDED_LINES + 1),
            ),
            "the Agent Skills specification recommends moving longer detail to referenced files",
            SKILL_PLUGIN_ID,
        ));
    }
}

fn check_codex_asset(source: &HarnessSource, findings: &mut Vec<Finding>) {
    let path = normalized_path(source);
    if path.ends_with("/.codex/config.toml") || path == ".codex/config.toml" {
        if let Err(error) = toml::from_str::<toml::Value>(&source.content) {
            findings.push(source_finding(
                source,
                Severity::Error,
                "HL110",
                "Codex project configuration is not valid TOML",
                (
                    error
                        .span()
                        .and_then(|span| line_for_offset(&source.content, span.start)),
                    error.span().map(|span| TextSpan {
                        start: span.start,
                        end: span.end,
                    }),
                ),
                error.to_string(),
                CODEX_ASSET_PLUGIN_ID,
            ));
        }
        return;
    }

    if path.contains("/.codex/agents/") || path.starts_with(".codex/agents/") {
        check_codex_agent(source, findings);
        return;
    }

    if path.contains("/.codex/rules/") || path.starts_with(".codex/rules/") {
        check_codex_rules(source, findings);
    }
}

fn check_codex_agent(source: &HarnessSource, findings: &mut Vec<Finding>) {
    let value = match toml::from_str::<toml::Value>(&source.content) {
        Ok(value) => value,
        Err(error) => {
            findings.push(source_finding(
                source,
                Severity::Error,
                "HL110",
                "Codex custom agent is not valid TOML",
                (
                    error
                        .span()
                        .and_then(|span| line_for_offset(&source.content, span.start)),
                    error.span().map(|span| TextSpan {
                        start: span.start,
                        end: span.end,
                    }),
                ),
                error.to_string(),
                CODEX_ASSET_PLUGIN_ID,
            ));
            return;
        }
    };

    for (field, rule_id) in [
        ("name", "HL111"),
        ("description", "HL112"),
        ("developer_instructions", "HL113"),
    ] {
        if value
            .get(field)
            .and_then(toml::Value::as_str)
            .is_some_and(|text| !text.trim().is_empty())
        {
            continue;
        }
        findings.push(toml_field_finding(
            source,
            rule_id,
            format!("Codex custom agent requires a non-empty {field} string"),
            field,
            "OpenAI requires name, description, and developer_instructions in every standalone custom agent",
        ));
    }
}

fn check_codex_rules(source: &HarnessSource, findings: &mut Vec<Finding>) {
    if source.content.trim().is_empty() || !source.content.contains("prefix_rule(") {
        findings.push(source_finding(
            source,
            Severity::Error,
            "HL114",
            "Codex rules file contains no prefix_rule declaration",
            (Some(1), line_span(&source.content, 1)),
            "Codex .rules files declare command policy with prefix_rule(...) calls",
            CODEX_ASSET_PLUGIN_ID,
        ));
        return;
    }

    for (line, start, text) in source_lines(&source.content) {
        let trimmed = text.trim();
        let Some(value) = trimmed
            .strip_prefix("decision")
            .and_then(|rest| rest.trim_start().strip_prefix('='))
            .map(|rest| rest.trim().trim_end_matches(',').trim_matches(['\'', '"']))
        else {
            continue;
        };
        if matches!(value, "allow" | "prompt" | "forbidden") {
            continue;
        }
        findings.push(source_finding(
            source,
            Severity::Error,
            "HL115",
            format!("Unknown Codex rule decision “{value}”"),
            (Some(line), Some(full_line_span(start, text))),
            "decision must be allow, prompt, or forbidden",
            CODEX_ASSET_PLUGIN_ID,
        ));
    }
}

fn convention_output(
    findings: Vec<Finding>,
    checked: usize,
    score_name: &str,
    metric_name: &str,
    plugin_id: &str,
    method: ScoreMethod,
    assumption: &str,
) -> PluginOutput {
    let finding_count = findings.len();
    let mut score = Score::new(
        score_name,
        ScoreCategory::Quality,
        method,
        f64::from(finding_count == 0),
        1.0,
        if finding_count == 0 {
            "No convention problems found"
        } else {
            "One or more convention problems found"
        },
        plugin_id,
    )
    .expect("built-in score is normalized");
    score.sample_size = Some(checked);
    score
        .evidence
        .insert("assumption".to_owned(), assumption.to_owned());
    score
        .evidence
        .insert("finding_count".to_owned(), finding_count.to_string());

    PluginOutput {
        findings,
        metrics: vec![Metric {
            name: metric_name.to_owned(),
            value: finding_count as f64,
            unit: Some("count".to_owned()),
            path: None,
            reference: None,
            source: plugin_id.to_owned(),
        }],
        scores: vec![score],
    }
}

fn source_finding(
    source: &HarnessSource,
    severity: Severity,
    rule_id: &str,
    message: impl Into<String>,
    location: (Option<usize>, Option<TextSpan>),
    evidence: impl Into<String>,
    plugin_id: &str,
) -> Finding {
    Finding {
        severity,
        rule_id: rule_id.to_owned(),
        message: message.into(),
        path: Some(source.path.clone()),
        line: location.0,
        span: location.1,
        evidence: Some(evidence.into()),
        related: Vec::new(),
        source: plugin_id.to_owned(),
    }
}

fn field_finding(
    source: &HarnessSource,
    rule_id: &str,
    message: impl Into<String>,
    field: &str,
    evidence: &str,
) -> Finding {
    yaml_field_finding(
        source,
        Severity::Error,
        rule_id,
        message,
        field,
        evidence,
        SKILL_PLUGIN_ID,
    )
}

fn yaml_field_finding(
    source: &HarnessSource,
    severity: Severity,
    rule_id: &str,
    message: impl Into<String>,
    field: &str,
    evidence: &str,
    plugin_id: &str,
) -> Finding {
    let (line, span) = field_location(&source.content, field).unwrap_or_else(|| {
        (
            1,
            line_span(&source.content, 1).unwrap_or(TextSpan { start: 0, end: 0 }),
        )
    });
    source_finding(
        source,
        severity,
        rule_id,
        message,
        (Some(line), Some(span)),
        evidence,
        plugin_id,
    )
}

fn toml_field_finding(
    source: &HarnessSource,
    rule_id: &str,
    message: impl Into<String>,
    field: &str,
    evidence: &str,
) -> Finding {
    let prefix = format!("{field} ");
    let location = source_lines(&source.content)
        .find(|(_, _, line)| line.trim_start().starts_with(&prefix))
        .map(|(line, start, text)| (line, full_line_span(start, text)))
        .unwrap_or_else(|| {
            (
                1,
                line_span(&source.content, 1).unwrap_or(TextSpan { start: 0, end: 0 }),
            )
        });
    source_finding(
        source,
        Severity::Error,
        rule_id,
        message,
        (Some(location.0), Some(location.1)),
        evidence,
        CODEX_ASSET_PLUGIN_ID,
    )
}

struct Frontmatter<'a> {
    value: Value,
    body: &'a str,
    body_start: usize,
    body_line: usize,
}

struct FrontmatterError {
    line: usize,
    message: String,
}

fn parse_frontmatter(content: &str) -> Result<Frontmatter<'_>, FrontmatterError> {
    let mut lines = source_lines(content);
    let Some((_, first_start, first)) = lines.next() else {
        return Err(FrontmatterError {
            line: 1,
            message: "file is empty".to_owned(),
        });
    };
    if first.trim() != "---" {
        return Err(FrontmatterError {
            line: 1,
            message: "frontmatter must start with --- on the first line".to_owned(),
        });
    }
    let yaml_start = first_start + first.len();
    for (line_number, line_start, line) in lines {
        if line.trim() != "---" {
            continue;
        }
        let yaml = &content[yaml_start..line_start];
        let body_start = line_start + line.len();
        let value = serde_yaml::from_str(yaml).map_err(|error| FrontmatterError {
            line: error.location().map_or(1, |location| location.line() + 1),
            message: error.to_string(),
        })?;
        return Ok(Frontmatter {
            value,
            body: &content[body_start..],
            body_start,
            body_line: line_number + 1,
        });
    }
    Err(FrontmatterError {
        line: 1,
        message: "frontmatter has no closing --- delimiter".to_owned(),
    })
}

fn string_field<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value
        .as_mapping()?
        .get(Value::String(field.to_owned()))?
        .as_str()
}

fn valid_skill_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn contains_phrase(line: &str, phrase: &str) -> bool {
    line.match_indices(phrase).any(|(start, matched)| {
        let end = start + matched.len();
        let before = line[..start].chars().next_back();
        let after = line[end..].chars().next();
        before.is_none_or(|character| !character.is_alphanumeric())
            && after.is_none_or(|character| !character.is_alphanumeric())
    })
}

fn unclosed_fence(content: &str) -> Option<(usize, TextSpan)> {
    let mut open = None;
    for (line_number, line_start, line) in source_lines(content) {
        let Some(marker) = fence_marker(line) else {
            continue;
        };
        match open {
            Some((open_marker, _, _)) if open_marker == marker => open = None,
            None => open = Some((marker, line_number, full_line_span(line_start, line))),
            Some(_) => {}
        }
    }
    open.map(|(_, line, span)| (line, span))
}

fn fence_marker(line: &str) -> Option<&'static str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("```") {
        Some("```")
    } else if trimmed.starts_with("~~~") {
        Some("~~~")
    } else {
        None
    }
}

fn field_location(content: &str, field: &str) -> Option<(usize, TextSpan)> {
    let prefix = format!("{field}:");
    source_lines(content)
        .take_while(|(line, _, text)| *line == 1 || text.trim() != "---")
        .find(|(_, _, line)| line.trim_start().starts_with(&prefix))
        .map(|(line, start, text)| (line, full_line_span(start, text)))
}

fn line_span(content: &str, target: usize) -> Option<TextSpan> {
    source_lines(content)
        .find(|(line, _, _)| *line == target)
        .map(|(_, start, text)| full_line_span(start, text))
}

fn line_for_offset(content: &str, offset: usize) -> Option<usize> {
    source_lines(content)
        .find(|(_, start, text)| offset < start + text.len())
        .map(|(line, _, _)| line)
        .or_else(|| (!content.is_empty()).then_some(content.lines().count().max(1)))
}

fn full_line_span(start: usize, line: &str) -> TextSpan {
    TextSpan {
        start,
        end: start + line.trim_end_matches(['\r', '\n']).len(),
    }
}

fn source_lines(content: &str) -> impl Iterator<Item = (usize, usize, &str)> {
    let mut offset = 0_usize;
    content
        .split_inclusive('\n')
        .enumerate()
        .map(move |(index, line)| {
            let start = offset;
            offset += line.len();
            (index + 1, start, line)
        })
}

fn file_name(source: &HarnessSource) -> Option<&str> {
    source.path.file_name().and_then(|name| name.to_str())
}

fn is_markdown_source(source: &HarnessSource) -> bool {
    source
        .path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension, "md" | "mdc"))
}

fn is_codex_asset(source: &HarnessSource) -> bool {
    let path = normalized_path(source);
    path == ".codex/config.toml"
        || path.ends_with("/.codex/config.toml")
        || path.starts_with(".codex/agents/")
        || path.contains("/.codex/agents/")
        || path.starts_with(".codex/rules/")
        || path.contains("/.codex/rules/")
}

fn normalized_path(source: &HarnessSource) -> String {
    source.path.to_string_lossy().replace('\\', "/")
}

fn is_claude_agent(source: &HarnessSource) -> bool {
    let path = normalized_path(source);
    (path.starts_with(".claude/agents/") || path.contains("/.claude/agents/"))
        && file_name(source).is_some_and(|name| name.ends_with(".md"))
}

fn is_github_agent(source: &HarnessSource) -> bool {
    let path = normalized_path(source);
    (path.starts_with(".github/agents/") || path.contains("/.github/agents/"))
        && file_name(source).is_some_and(|name| name.ends_with(".agent.md"))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::*;
    use crate::{HarnessLensConfig, PluginContext};

    fn source(path: &str, kind: HarnessSourceKind, content: String) -> HarnessSource {
        let path = PathBuf::from(path);
        HarnessSource {
            scope: path.parent().unwrap_or(Path::new("")).to_owned(),
            path,
            kind,
            content,
        }
    }

    fn context<'a>(
        sources: &'a [HarnessSource],
        config: &'a HarnessLensConfig,
    ) -> PluginContext<'a> {
        static OPTIONS: std::sync::LazyLock<BTreeMap<String, String>> =
            std::sync::LazyLock::new(BTreeMap::new);
        PluginContext {
            config,
            sources,
            options: &OPTIONS,
        }
    }

    #[test]
    fn instruction_checks_cover_empty_vague_and_unclosed_content() {
        let sources = [
            source("AGENTS.md", HarnessSourceKind::Agents, String::new()),
            source(
                "CLAUDE.md",
                HarnessSourceKind::Instructions,
                "- Format code properly.\n```text\n".to_owned(),
            ),
        ];
        let config = HarnessLensConfig::default();
        let output = InstructionConventionsPlugin
            .analyze(&context(&sources, &config))
            .unwrap();

        assert_eq!(
            output
                .findings
                .iter()
                .map(|finding| finding.rule_id.as_str())
                .collect::<Vec<_>>(),
            ["HL040", "HL043", "HL044"]
        );
    }

    #[test]
    fn instruction_checks_provider_size_guidance() {
        let sources = [
            source(
                "AGENTS.md",
                HarnessSourceKind::Agents,
                "a".repeat(CODEX_DEFAULT_BYTES + 1),
            ),
            source(
                "CLAUDE.md",
                HarnessSourceKind::Instructions,
                "specific instruction\n".repeat(CLAUDE_TARGET_LINES + 1),
            ),
        ];
        let config = HarnessLensConfig::default();
        let output = InstructionConventionsPlugin
            .analyze(&context(&sources, &config))
            .unwrap();

        assert!(
            output
                .findings
                .iter()
                .any(|finding| finding.rule_id == "HL041")
        );
        assert!(
            output
                .findings
                .iter()
                .any(|finding| finding.rule_id == "HL042")
        );
    }

    #[test]
    fn copilot_path_rules_require_apply_to_frontmatter() {
        let sources = [source(
            ".github/instructions/rust.instructions.md",
            HarnessSourceKind::Rules,
            "---\ndescription: Rust\n---\nUse cargo.\n".to_owned(),
        )];
        let config = HarnessLensConfig::default();
        let output = InstructionConventionsPlugin
            .analyze(&context(&sources, &config))
            .unwrap();

        assert_eq!(output.findings[0].rule_id, "HL045");
    }

    #[test]
    fn valid_portable_skill_passes() {
        let sources = [source(
            ".agents/skills/review-code/SKILL.md",
            HarnessSourceKind::Skills,
            "---\nname: review-code\ndescription: Review code when the user asks for a review.\n---\n\nReview the requested changes.\n".to_owned(),
        )];
        let config = HarnessLensConfig::default();
        let output = SkillConventionsPlugin
            .analyze(&context(&sources, &config))
            .unwrap();

        assert!(output.findings.is_empty());
    }

    #[test]
    fn invalid_skill_reports_schema_name_description_and_body() {
        let sources = [source(
            ".claude/skills/review-code/SKILL.md",
            HarnessSourceKind::Skills,
            "---\nname: Review--Code\ndescription: \n---\n".to_owned(),
        )];
        let config = HarnessLensConfig::default();
        let output = SkillConventionsPlugin
            .analyze(&context(&sources, &config))
            .unwrap();

        assert_eq!(
            output
                .findings
                .iter()
                .map(|finding| finding.rule_id.as_str())
                .collect::<Vec<_>>(),
            ["HL101", "HL103", "HL104"]
        );
    }

    #[test]
    fn skill_name_must_match_directory() {
        let sources = [source(
            ".agents/skills/review-code/SKILL.md",
            HarnessSourceKind::Skills,
            "---\nname: inspect-code\ndescription: Inspect code when requested.\n---\nInspect it.\n".to_owned(),
        )];
        let config = HarnessLensConfig::default();
        let output = SkillConventionsPlugin
            .analyze(&context(&sources, &config))
            .unwrap();

        assert_eq!(output.findings[0].rule_id, "HL102");
    }

    #[test]
    fn codex_config_and_agent_toml_are_validated() {
        let sources = [
            source(
                ".codex/config.toml",
                HarnessSourceKind::Configuration,
                "model = [\n".to_owned(),
            ),
            source(
                ".codex/agents/reviewer.toml",
                HarnessSourceKind::Agents,
                "name = \"reviewer\"\ndescription = \"\"\n".to_owned(),
            ),
        ];
        let config = HarnessLensConfig::default();
        let output = CodexAssetConventionsPlugin
            .analyze(&context(&sources, &config))
            .unwrap();

        assert_eq!(
            output
                .findings
                .iter()
                .map(|finding| finding.rule_id.as_str())
                .collect::<Vec<_>>(),
            ["HL110", "HL112", "HL113"],
            "{:#?}",
            output.findings
        );
    }

    #[test]
    fn codex_rule_decisions_are_checked() {
        let sources = [source(
            ".codex/rules/default.rules",
            HarnessSourceKind::Rules,
            "prefix_rule(\n  pattern = [\"git\"],\n  decision = \"deny\",\n)\n".to_owned(),
        )];
        let config = HarnessLensConfig::default();
        let output = CodexAssetConventionsPlugin
            .analyze(&context(&sources, &config))
            .unwrap();

        assert_eq!(output.findings[0].rule_id, "HL115");
    }

    #[test]
    fn markdown_agent_profiles_follow_provider_schemas() {
        let sources = [
            source(
                ".claude/agents/reviewer.md",
                HarnessSourceKind::Agents,
                "---\nname: Reviewer\ndescription: Review code\n---\n\nDo reviews.\n".to_owned(),
            ),
            source(
                ".github/agents/helper.agent.md",
                HarnessSourceKind::Agents,
                "---\nname: Helper\n---\n".to_owned(),
            ),
        ];
        let config = HarnessLensConfig::default();
        let output = InstructionConventionsPlugin
            .analyze(&context(&sources, &config))
            .unwrap();

        assert_eq!(
            output
                .findings
                .iter()
                .map(|finding| finding.rule_id.as_str())
                .collect::<Vec<_>>(),
            ["HL122", "HL121", "HL123"]
        );
    }
}
