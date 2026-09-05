// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

use crate::{
    Finding, HarnessSource, Metric, Plugin, PluginContext, PluginError, PluginMetadata,
    PluginOutput, Score, ScoreCategory, ScoreMethod, Severity,
};

const EVALUATION_PLUGIN_ID: &str = "harness-lens.evaluation";

/// Emits source-size, token-budget, and optional invocation-cost measurements.
pub(crate) struct EvaluationPlugin;

impl Plugin for EvaluationPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: EVALUATION_PLUGIN_ID.to_owned(),
            name: "Source size and token cost evaluation".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            capabilities: vec!["metrics.tokens".to_owned(), "metrics.cost".to_owned()],
            default_enabled: true,
        }
    }

    fn analyze(&self, context: &PluginContext<'_>) -> Result<PluginOutput, PluginError> {
        let mut findings = Vec::new();
        let mut metrics = Vec::new();
        let mut total_bytes = 0_u64;
        let mut total_tokens = 0_usize;
        let mut max_bytes = 0_u64;
        let mut max_tokens = 0_usize;
        let mut large_sources = 0_usize;
        let mut over_elaborated_sources = 0_usize;
        let valid_rate = context
            .config
            .evaluation
            .input_cost_per_million_tokens
            .filter(|rate| rate.is_finite() && *rate >= 0.0);

        for source in context.sources {
            let bytes = source.content.len() as u64;
            let tokens = estimate_tokens(&source.content);
            let lines = source_lines(&source.content).count();
            let paragraphs = count_paragraphs(&source.content);
            total_bytes += bytes;
            total_tokens += tokens;
            max_bytes = max_bytes.max(bytes);
            max_tokens = max_tokens.max(tokens);

            metrics.push(source_metric(
                "harness.source.bytes",
                bytes as f64,
                "bytes",
                source,
            ));
            metrics.push(source_metric(
                "harness.source.estimated_tokens",
                tokens as f64,
                "tokens",
                source,
            ));
            metrics.push(source_metric(
                "harness.source.lines",
                lines as f64,
                "count",
                source,
            ));
            metrics.push(source_metric(
                "harness.source.paragraphs",
                paragraphs as f64,
                "count",
                source,
            ));
            if let Some(rate) = valid_rate {
                let per_invocation = tokens as f64 * rate / 1_000_000.0;
                let total = per_invocation * context.config.evaluation.invocations.max(1) as f64;
                let reference = context.config.evaluation.cost_reference.clone();
                metrics.push(Metric {
                    name: "harness.source.input_cost_per_invocation".to_owned(),
                    value: per_invocation,
                    unit: Some(format!("{}/invocation", context.config.evaluation.currency)),
                    path: Some(source.path.clone()),
                    reference: reference.clone(),
                    source: EVALUATION_PLUGIN_ID.to_owned(),
                });
                metrics.push(Metric {
                    name: "harness.source.input_cost_total".to_owned(),
                    value: total,
                    unit: Some(context.config.evaluation.currency.clone()),
                    path: Some(source.path.clone()),
                    reference,
                    source: EVALUATION_PLUGIN_ID.to_owned(),
                });
            }

            if bytes > context.config.evaluation.max_source_bytes {
                large_sources += 1;
                findings.push(Finding {
                    severity: Severity::Warning,
                    rule_id: "HL050".to_owned(),
                    message: format!(
                        "Harness source is too large: {bytes} bytes exceeds {}",
                        context.config.evaluation.max_source_bytes
                    ),
                    path: Some(source.path.clone()),
                    line: Some(1),
                    span: None,
                    evidence: Some(
                        "soft source-size budget; configure evaluation.max_source_bytes".to_owned(),
                    ),
                    related: Vec::new(),
                    source: EVALUATION_PLUGIN_ID.to_owned(),
                });
            }
            if tokens > context.config.evaluation.max_source_tokens {
                over_elaborated_sources += 1;
                findings.push(Finding {
                    severity: Severity::Warning,
                    rule_id: "HL051".to_owned(),
                    message: format!(
                        "Harness source is over-elaborated: {tokens} estimated tokens exceeds {}",
                        context.config.evaluation.max_source_tokens
                    ),
                    path: Some(source.path.clone()),
                    line: Some(1),
                    span: None,
                    evidence: Some(
                        "soft token-budget heuristic; tokens are estimated as Unicode scalar count / 4"
                            .to_owned(),
                    ),
                    related: Vec::new(),
                    source: EVALUATION_PLUGIN_ID.to_owned(),
                });
            }
        }

        metrics.extend([
            aggregate_metric("harness.total_source_bytes", total_bytes as f64, "bytes"),
            aggregate_metric(
                "harness.total_estimated_tokens",
                total_tokens as f64,
                "tokens/invocation",
            ),
            aggregate_metric("harness.max_source_bytes", max_bytes as f64, "bytes"),
            aggregate_metric(
                "harness.max_source_estimated_tokens",
                max_tokens as f64,
                "tokens",
            ),
            aggregate_metric("harness.large_sources", large_sources as f64, "count"),
            aggregate_metric(
                "harness.over_elaborated_sources",
                over_elaborated_sources as f64,
                "count",
            ),
            aggregate_metric(
                "harness.invocations",
                context.config.evaluation.invocations.max(1) as f64,
                "count",
            ),
        ]);

        if let Some(rate) = context.config.evaluation.input_cost_per_million_tokens {
            if rate.is_finite() && rate >= 0.0 {
                let invocations = context.config.evaluation.invocations.max(1) as f64;
                let per_invocation = total_tokens as f64 * rate / 1_000_000.0;
                let total = per_invocation * invocations;
                let reference = context.config.evaluation.cost_reference.clone();
                metrics.push(Metric {
                    name: "harness.input_cost_per_invocation".to_owned(),
                    value: per_invocation,
                    unit: Some(format!("{}/invocation", context.config.evaluation.currency)),
                    path: None,
                    reference: reference.clone(),
                    source: EVALUATION_PLUGIN_ID.to_owned(),
                });
                metrics.push(Metric {
                    name: "harness.input_cost_total".to_owned(),
                    value: total,
                    unit: Some(context.config.evaluation.currency.clone()),
                    path: None,
                    reference,
                    source: EVALUATION_PLUGIN_ID.to_owned(),
                });
            } else {
                findings.push(Finding {
                    severity: Severity::Error,
                    rule_id: "HL052".to_owned(),
                    message: "Input token price must be finite and non-negative".to_owned(),
                    path: None,
                    line: None,
                    span: None,
                    evidence: Some(
                        "cost was not calculated because the caller supplied an invalid price"
                            .to_owned(),
                    ),
                    related: Vec::new(),
                    source: EVALUATION_PLUGIN_ID.to_owned(),
                });
            }
        }

        let budget_findings = large_sources + over_elaborated_sources;
        let mut score = Score::new(
            "harness.resource_budget",
            ScoreCategory::Performance,
            ScoreMethod::Heuristic,
            f64::from(budget_findings == 0),
            1.0,
            if budget_findings == 0 {
                "All harness sources are within configured size and token budgets"
            } else {
                "One or more harness sources exceed configured size or token budgets"
            },
            EVALUATION_PLUGIN_ID,
        )
        .expect("built-in score is normalized");
        score.sample_size = Some(context.sources.len());
        score.evidence.insert(
            "token_assumption".to_owned(),
            "estimated tokens = ceil(Unicode scalar count / 4); actual tokenizer varies by model"
                .to_owned(),
        );
        score.evidence.insert(
            "cost_assumption".to_owned(),
            "input injection cost only; total = per-invocation cost × configured invocations"
                .to_owned(),
        );
        score.evidence.insert(
            "cost_reference".to_owned(),
            context
                .config
                .evaluation
                .cost_reference
                .clone()
                .unwrap_or_else(|| "not supplied".to_owned()),
        );

        Ok(PluginOutput {
            findings,
            metrics,
            scores: vec![score],
        })
    }
}

fn source_metric(name: &str, value: f64, unit: &str, source: &HarnessSource) -> Metric {
    Metric {
        name: name.to_owned(),
        value,
        unit: Some(unit.to_owned()),
        path: Some(source.path.clone()),
        reference: None,
        source: EVALUATION_PLUGIN_ID.to_owned(),
    }
}

fn aggregate_metric(name: &str, value: f64, unit: &str) -> Metric {
    Metric {
        name: name.to_owned(),
        value,
        unit: Some(unit.to_owned()),
        path: None,
        reference: None,
        source: EVALUATION_PLUGIN_ID.to_owned(),
    }
}

fn estimate_tokens(content: &str) -> usize {
    let characters = content.chars().count();
    characters.div_ceil(4)
}

fn count_paragraphs(content: &str) -> usize {
    let mut count = 0_usize;
    let mut in_paragraph = false;
    let mut in_fence = false;
    for (_, _, raw) in source_lines(content) {
        if is_fence(raw) {
            if in_paragraph {
                count += 1;
                in_paragraph = false;
            }
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if raw.trim().is_empty() {
            if in_paragraph {
                count += 1;
                in_paragraph = false;
            }
        } else {
            in_paragraph = true;
        }
    }
    if in_paragraph {
        count += 1;
    }
    count
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
    use crate::{EvaluationConfig, HarnessLensConfig, HarnessSourceKind, PluginContext};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn source(path: &str, content: &str) -> HarnessSource {
        HarnessSource {
            path: PathBuf::from(path),
            kind: HarnessSourceKind::Agents,
            scope: PathBuf::from("/workspace"),
            content: content.to_owned(),
        }
    }

    #[test]
    fn calculates_cost_per_invocation_and_total() {
        let sources = [source("AGENTS.md", "Run the complete test suite.\n")];
        let config = HarnessLensConfig {
            evaluation: EvaluationConfig {
                invocations: 10,
                input_cost_per_million_tokens: Some(2.0),
                cost_reference: Some("test-model/input".to_owned()),
                max_source_bytes: 1,
                max_source_tokens: 1,
                ..EvaluationConfig::default()
            },
            ..HarnessLensConfig::default()
        };
        static OPTIONS: std::sync::LazyLock<BTreeMap<String, String>> =
            std::sync::LazyLock::new(BTreeMap::new);
        let output = EvaluationPlugin
            .analyze(&PluginContext {
                config: &config,
                sources: &sources,
                options: &OPTIONS,
            })
            .unwrap();

        let per_invocation = output
            .metrics
            .iter()
            .find(|metric| metric.name == "harness.input_cost_per_invocation")
            .expect("per-invocation cost metric");
        let total = output
            .metrics
            .iter()
            .find(|metric| metric.name == "harness.input_cost_total")
            .expect("total cost metric");
        assert!(per_invocation.value > 0.0);
        assert_eq!(total.value, per_invocation.value * 10.0);
        assert_eq!(
            per_invocation.reference.as_deref(),
            Some("test-model/input")
        );
        assert!(
            output
                .findings
                .iter()
                .any(|finding| finding.rule_id == "HL050")
        );
        assert!(
            output
                .findings
                .iter()
                .any(|finding| finding.rule_id == "HL051")
        );
    }
}
