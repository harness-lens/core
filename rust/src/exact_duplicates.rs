// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::{
    Finding, FindingLocation, HarnessSource, Metric, Plugin, PluginContext, PluginError,
    PluginMetadata, PluginOutput, Score, ScoreCategory, ScoreMethod, Severity, TextSpan,
};

pub(crate) const EXACT_DUPLICATE_PLUGIN_ID: &str = "harness-lens.exact-duplicates";
pub(crate) const EXACT_DUPLICATE_NORMALIZATION: &str = "Unicode case-fold approximation via lowercase; trim and collapse whitespace; remove Markdown heading/list markers and emphasis backticks; ignore fenced code";

/// Finds exact normalized instruction lines and multi-line paragraphs repeated
/// in overlapping harness scopes.
pub(crate) struct ExactDuplicatePlugin;

impl Plugin for ExactDuplicatePlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: EXACT_DUPLICATE_PLUGIN_ID.to_owned(),
            name: "Exact duplicate lines and paragraphs".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            capabilities: vec!["text.exact-duplicates".to_owned()],
            default_enabled: true,
        }
    }

    fn analyze(&self, context: &PluginContext<'_>) -> Result<PluginOutput, PluginError> {
        let mut groups: BTreeMap<(UnitKind, String), Vec<TextUnit>> = BTreeMap::new();
        let mut unit_count = 0_usize;

        for source in context.sources {
            for unit in extract_units(source) {
                if unit.normalized.len() < 8 {
                    continue;
                }
                unit_count += 1;
                groups
                    .entry((unit.kind, unit.normalized.clone()))
                    .or_default()
                    .push(unit);
            }
        }

        let mut findings = Vec::new();
        for units in groups.values() {
            for (index, current) in units.iter().enumerate() {
                let Some(previous) = units[..index]
                    .iter()
                    .rev()
                    .find(|previous| scopes_overlap(&current.scope, &previous.scope))
                else {
                    continue;
                };

                let kind = match current.kind {
                    UnitKind::Line => "line",
                    UnitKind::Paragraph => "paragraph",
                };
                findings.push(Finding {
                    severity: Severity::Warning,
                    rule_id: "HL032".to_owned(),
                    message: format!(
                        "Exact duplicate {kind} repeats {}:{}",
                        previous.path.display(),
                        previous.line
                    ),
                    path: Some(current.path.clone()),
                    line: Some(current.line),
                    span: Some(current.span),
                    evidence: Some(format!(
                        "normalized {kind} matches exactly; assumption: {EXACT_DUPLICATE_NORMALIZATION}"
                    )),
                    related: vec![FindingLocation {
                        path: previous.path.clone(),
                        line: Some(previous.line),
                        span: Some(previous.span),
                    }],
                    source: EXACT_DUPLICATE_PLUGIN_ID.to_owned(),
                });
            }
        }
        findings.sort_by(|left, right| {
            left.path
                .cmp(&right.path)
                .then(left.line.cmp(&right.line))
                .then(left.rule_id.cmp(&right.rule_id))
        });

        let finding_count = findings.len();
        let mut score = Score::new(
            "harness.exact_duplicates_free",
            ScoreCategory::Quality,
            ScoreMethod::Heuristic,
            f64::from(finding_count == 0),
            1.0,
            if finding_count == 0 {
                "No exact duplicate lines or paragraphs found"
            } else {
                "Exact duplicate lines or paragraphs found"
            },
            EXACT_DUPLICATE_PLUGIN_ID,
        )
        .expect("built-in score is normalized");
        score.sample_size = Some(unit_count);
        score.evidence.insert(
            "assumption".to_owned(),
            EXACT_DUPLICATE_NORMALIZATION.to_owned(),
        );
        score
            .evidence
            .insert("finding_count".to_owned(), finding_count.to_string());

        Ok(PluginOutput {
            findings,
            metrics: vec![Metric {
                name: "harness.exact_duplicate_lines_or_paragraphs".to_owned(),
                value: finding_count as f64,
                unit: Some("count".to_owned()),
                path: None,
                reference: Some(EXACT_DUPLICATE_NORMALIZATION.to_owned()),
                source: EXACT_DUPLICATE_PLUGIN_ID.to_owned(),
            }],
            scores: vec![score],
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum UnitKind {
    Line,
    Paragraph,
}

#[derive(Clone, Debug)]
struct TextUnit {
    normalized: String,
    kind: UnitKind,
    path: PathBuf,
    scope: PathBuf,
    line: usize,
    span: TextSpan,
}

#[derive(Clone, Debug)]
struct ParagraphLine {
    line: usize,
    start: usize,
    end: usize,
    text: String,
}

fn extract_units(source: &HarnessSource) -> Vec<TextUnit> {
    let mut units = Vec::new();
    let mut paragraph = Vec::new();
    let mut in_fence = false;

    for (line, start, raw) in source_lines(&source.content) {
        if is_fence(raw) {
            flush_paragraph(&mut units, &mut paragraph, source);
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        let text = raw.trim_end_matches(['\r', '\n']);
        if text.trim().is_empty() {
            flush_paragraph(&mut units, &mut paragraph, source);
            continue;
        }

        if let Some(normalized) = normalize_exact(text) {
            units.push(TextUnit {
                normalized,
                kind: UnitKind::Line,
                path: source.path.clone(),
                scope: source.scope.clone(),
                line,
                span: TextSpan {
                    start,
                    end: start + text.len(),
                },
            });
            paragraph.push(ParagraphLine {
                line,
                start,
                end: start + text.len(),
                text: text.to_owned(),
            });
        }
    }
    flush_paragraph(&mut units, &mut paragraph, source);
    units
}

fn flush_paragraph(
    units: &mut Vec<TextUnit>,
    paragraph: &mut Vec<ParagraphLine>,
    source: &HarnessSource,
) {
    if paragraph.len() > 1 {
        let text = paragraph
            .iter()
            .map(|line| line.text.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        if let Some(normalized) = normalize_exact(&text) {
            let first = paragraph.first().expect("paragraph is not empty");
            let last = paragraph.last().expect("paragraph is not empty");
            units.push(TextUnit {
                normalized,
                kind: UnitKind::Paragraph,
                path: source.path.clone(),
                scope: source.scope.clone(),
                line: first.line,
                span: TextSpan {
                    start: first.start,
                    end: last.end,
                },
            });
        }
    }
    paragraph.clear();
}

fn normalize_exact(text: &str) -> Option<String> {
    let mut text = text.trim();
    while let Some(rest) = text.strip_prefix('#') {
        text = rest.trim_start();
    }
    if let Some(rest) = text
        .strip_prefix("- ")
        .or_else(|| text.strip_prefix("* "))
        .or_else(|| text.strip_prefix("+ "))
    {
        text = rest.trim_start();
    } else if let Some(index) = text.find(". ")
        && text[..index]
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        text = text[index + 2..].trim_start();
    }

    let normalized = text
        .to_lowercase()
        .replace(['`', '*'], "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!normalized.is_empty()).then_some(normalized)
}

fn scopes_overlap(left: &Path, right: &Path) -> bool {
    left == right || left.starts_with(right) || right.starts_with(left)
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

fn is_fence(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HarnessLensConfig, HarnessSourceKind, PluginContext};
    use std::collections::BTreeMap;

    fn source(path: &str, scope: &str, content: &str) -> HarnessSource {
        HarnessSource {
            path: path.into(),
            kind: HarnessSourceKind::Agents,
            scope: scope.into(),
            content: content.to_owned(),
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
    fn reports_exact_duplicate_line_with_both_locations() {
        let sources = [
            source(
                "AGENTS.md",
                "/workspace",
                "Adoption, rejection, assumptions, and source links.\n",
            ),
            source(
                "nested/AGENTS.md",
                "/workspace/nested",
                "- adoption,   rejection, assumptions, and source links.\n",
            ),
        ];
        let config = HarnessLensConfig::default();
        let output = ExactDuplicatePlugin
            .analyze(&context(&sources, &config))
            .unwrap();

        assert_eq!(output.findings.len(), 1);
        assert_eq!(output.findings[0].rule_id, "HL032");
        assert_eq!(
            output.findings[0].path,
            Some(PathBuf::from("nested/AGENTS.md"))
        );
        assert_eq!(output.findings[0].line, Some(1));
        assert_eq!(
            output.findings[0].related[0].path,
            PathBuf::from("AGENTS.md")
        );
        assert!(
            output.findings[0]
                .evidence
                .as_deref()
                .is_some_and(|evidence| evidence.contains("assumption:"))
        );
    }

    #[test]
    fn reports_duplicate_multiline_paragraph() {
        let sources = [source(
            "AGENTS.md",
            ".",
            "Use the repository formatter.\nRun the complete test suite.\n\nuse the repository formatter.\nrun the complete test suite.\n",
        )];
        let config = HarnessLensConfig::default();
        let output = ExactDuplicatePlugin
            .analyze(&context(&sources, &config))
            .unwrap();

        assert!(output.findings.iter().any(|finding| {
            finding
                .evidence
                .as_deref()
                .is_some_and(|evidence| evidence.contains("paragraph"))
        }));
    }

    #[test]
    fn ignores_fenced_code_and_disjoint_scopes() {
        let sources = [
            source("a/AGENTS.md", "a", "```text\nRepeat this line.\n```\n"),
            source("b/AGENTS.md", "b", "Repeat this line.\n"),
        ];
        let config = HarnessLensConfig::default();
        let output = ExactDuplicatePlugin
            .analyze(&context(&sources, &config))
            .unwrap();

        assert!(output.findings.is_empty());
    }
}
