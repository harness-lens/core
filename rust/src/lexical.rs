// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

//! Bounded lexical edit similarity. No semantic inference or score aggregation.

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use unicode_casefold::UnicodeCaseFold;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
use unicode_properties::{GeneralCategoryGroup, UnicodeGeneralCategory};

use crate::{HarnessSource, ScoreMethod, TextSpan};

/// NFC, full non-Turkic case folding, then NFC; pinned library versions.
/// Letter boundaries use unicode-properties 0.1.3's pinned Unicode 16 table.
pub const NORMALIZATION: &str = "nfc-0.1.24-full-casefold-0.2.0-nonturkic-nfc-0.1.24-v1";

/// Resource and acceptance policy. Invalid configuration yields no findings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LexicalConfig {
    /// Maximum accepted normalized distance (inclusive), in [0, 1].
    pub threshold: f64,
    /// Minimum original AND normalized Unicode scalar length.
    pub min_token_length: usize,
    /// Maximum original AND normalized scalar length, at most 256.
    pub max_token_length: usize,
    /// Maximum token occurrences, including short tokens, at most 4096.
    pub max_tokens: usize,
    /// Maximum visited candidate pairs, including exact pairs, at most 100000.
    pub max_pairs: usize,
    /// Maximum total source bytes, at most 4 MiB. Partial tokens are excluded.
    pub max_input_bytes: usize,
    /// Maximum sources, at most 4096; excess input is rejected before sorting.
    pub max_sources: usize,
}

impl Default for LexicalConfig {
    fn default() -> Self {
        Self {
            threshold: 0.2,
            min_token_length: 4,
            max_token_length: 64,
            max_tokens: 1024,
            max_pairs: 10_000,
            max_input_bytes: 256 * 1024,
            max_sources: 1024,
        }
    }
}

/// Content-free limitation observed during lexical analysis.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LexicalLimit {
    /// Invalid threshold or resource limits.
    InvalidConfiguration,
    /// Too many input sources.
    Sources,
    /// Duplicate paths prevent unambiguous local evidence.
    DuplicatePath,
    /// Input byte budget exhausted.
    InputBytes,
    /// A token exceeded the original or normalized length bound.
    TokenLength,
    /// Token occurrence budget exhausted.
    Tokens,
    /// Candidate pair budget exhausted.
    Pairs,
}

/// Original local location; no token text is retained.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LexicalEvidence {
    /// Source path supplied by the local adapter.
    pub path: PathBuf,
    /// Half-open UTF-8 byte offsets before normalization.
    pub span: TextSpan,
}

/// Explainable lexical candidate, not a synonym or redundancy assertion.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LexicalFinding {
    /// Always heuristic.
    pub method: ScoreMethod,
    /// Exactly two original UTF-8 evidence locations.
    pub evidence: [LexicalEvidence; 2],
    /// Edit distance / maximum normalized Unicode scalar length.
    pub normalized_distance: f64,
    /// Configured inclusive maximum distance.
    pub threshold: f64,
    /// Stable normalization algorithm and Unicode table identifier.
    pub normalization: String,
    /// Fixed, content-free interpretation limits.
    pub assumptions: Vec<String>,
    /// True only when the entire bounded analysis completed.
    pub complete: bool,
    /// Same limitations as the containing report.
    pub incomplete_reasons: Vec<LexicalLimit>,
}

/// Separate heuristic output. Never changes deterministic findings or scores.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LexicalReport {
    /// Candidates in path/span order.
    pub findings: Vec<LexicalFinding>,
    /// Whether all requested lexical input was compared.
    pub complete: bool,
    /// Stable ordered limitation codes.
    pub incomplete_reasons: Vec<LexicalLimit>,
    /// Number of token occurrences inspected, including suppressed short tokens.
    pub tokens_examined: usize,
    /// Number of candidate pairs visited, including exact matches.
    pub pairs_examined: usize,
}

impl Default for LexicalReport {
    fn default() -> Self {
        Self {
            findings: Vec::new(),
            complete: true,
            incomplete_reasons: Vec::new(),
            tokens_examined: 0,
            pairs_examined: 0,
        }
    }
}

struct Token {
    text: Vec<char>,
    evidence: LexicalEvidence,
}

/// Analyzes local source text with global hard bounds and deterministic ordering.
#[must_use]
pub fn analyze(sources: &[HarnessSource], config: &LexicalConfig) -> LexicalReport {
    let mut report = LexicalReport::default();
    let mut limits = BTreeSet::new();
    if !config.threshold.is_finite()
        || !(0.0..=1.0).contains(&config.threshold)
        || config.min_token_length == 0
        || config.min_token_length > config.max_token_length
        || !(1..=256).contains(&config.max_token_length)
        || !(1..=4096).contains(&config.max_tokens)
        || !(1..=100_000).contains(&config.max_pairs)
        || !(1..=4_194_304).contains(&config.max_input_bytes)
        || !(1..=4096).contains(&config.max_sources)
    {
        limits.insert(LexicalLimit::InvalidConfiguration);
    } else if sources.len() > config.max_sources {
        limits.insert(LexicalLimit::Sources);
    } else {
        let mut sources: Vec<_> = sources.iter().collect();
        sources.sort_by(|a, b| a.path.cmp(&b.path));
        if sources.windows(2).any(|pair| pair[0].path == pair[1].path) {
            limits.insert(LexicalLimit::DuplicatePath);
        } else {
            let mut tokens = Vec::new();
            let mut remaining = config.max_input_bytes;
            'sources: for source in sources {
                let mut end = remaining.min(source.content.len());
                while !source.content.is_char_boundary(end) {
                    end -= 1;
                }
                remaining -= end;
                let truncated = end < source.content.len();
                if truncated {
                    limits.insert(LexicalLimit::InputBytes);
                }
                let text = &source.content[..end];
                let mut start = None;
                for (offset, ch) in text.char_indices().chain(std::iter::once((end, ' '))) {
                    if offset < end
                        && (ch.general_category_group() == GeneralCategoryGroup::Letter
                            || (start.is_some() && is_combining_mark(ch)))
                    {
                        start.get_or_insert(offset);
                    } else if let Some(begin) = start.take() {
                        if offset == end && truncated {
                            break;
                        }
                        if report.tokens_examined == config.max_tokens {
                            limits.insert(LexicalLimit::Tokens);
                            break 'sources;
                        }
                        report.tokens_examined += 1;
                        let raw = &text[begin..offset];
                        let length = raw.chars().take(config.max_token_length + 1).count();
                        if length > config.max_token_length {
                            limits.insert(LexicalLimit::TokenLength);
                            continue;
                        }
                        if length < config.min_token_length {
                            continue;
                        }
                        let normalized: Vec<char> = raw
                            .nfc()
                            .case_fold()
                            .nfc()
                            .take(config.max_token_length + 1)
                            .collect();
                        if normalized.len() > config.max_token_length {
                            limits.insert(LexicalLimit::TokenLength);
                            continue;
                        }
                        if normalized.len() >= config.min_token_length {
                            tokens.push(Token {
                                text: normalized,
                                evidence: LexicalEvidence {
                                    path: source.path.clone(),
                                    span: TextSpan {
                                        start: begin,
                                        end: offset,
                                    },
                                },
                            });
                        }
                    }
                }
                if truncated {
                    break;
                }
            }
            'pairs: for (index, left) in tokens.iter().enumerate() {
                for right in &tokens[index + 1..] {
                    if report.pairs_examined == config.max_pairs {
                        limits.insert(LexicalLimit::Pairs);
                        break 'pairs;
                    }
                    report.pairs_examined += 1;
                    if left.text == right.text {
                        continue;
                    }
                    let length = left.text.len().max(right.text.len());
                    // Derive integer budget using the same division as acceptance;
                    // avoids rounding below an inclusive decimal boundary.
                    let budget = (0..=length)
                        .take_while(|d| *d as f64 / length as f64 <= config.threshold)
                        .last()
                        .unwrap_or(0);
                    if let Some(distance) = bounded_distance(&left.text, &right.text, budget) {
                        report.findings.push(LexicalFinding {
                            method: ScoreMethod::Heuristic,
                            evidence: [left.evidence.clone(), right.evidence.clone()],
                            normalized_distance: distance as f64 / length as f64,
                            threshold: config.threshold, normalization: NORMALIZATION.to_owned(),
                            assumptions: vec!["lexical proximity only; no synonym, intent, or redundancy inference".to_owned(), "Unicode scalar edits; locale-independent non-Turkic folding; false positives require local review".to_owned()],
                            complete: true, incomplete_reasons: Vec::new(),
                        });
                    }
                }
            }
        }
    }
    report.incomplete_reasons = limits.into_iter().collect();
    report.complete = report.incomplete_reasons.is_empty();
    for finding in &mut report.findings {
        finding.complete = report.complete;
        finding
            .incomplete_reasons
            .clone_from(&report.incomplete_reasons);
    }
    report
}

fn bounded_distance(left: &[char], right: &[char], budget: usize) -> Option<usize> {
    if left.len().abs_diff(right.len()) > budget {
        return None;
    }
    let unreachable = budget + 1;
    let mut previous: Vec<usize> = (0..=right.len()).map(|n| n.min(unreachable)).collect();
    let mut current = vec![unreachable; right.len() + 1];
    for (row, a) in left.iter().enumerate() {
        let i = row + 1;
        current.fill(unreachable);
        current[0] = i.min(unreachable);
        let start = i.saturating_sub(budget).max(1);
        let end = (i + budget).min(right.len());
        for j in start..=end {
            current[j] = (previous[j] + 1)
                .min(current[j - 1] + 1)
                .min(previous[j - 1] + usize::from(*a != right[j - 1]));
        }
        if current.iter().all(|n| *n > budget) {
            return None;
        }
        std::mem::swap(&mut previous, &mut current);
    }
    (previous[right.len()] <= budget).then_some(previous[right.len()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AnalysisEngine, HarnessLensConfig, HarnessSourceKind};

    fn source(path: &str, content: &str) -> HarnessSource {
        HarnessSource {
            path: path.into(),
            scope: "".into(),
            kind: HarnessSourceKind::Instructions,
            content: content.into(),
        }
    }
    fn run(content: &str, config: LexicalConfig) -> LexicalReport {
        analyze(&[source("a.md", content)], &config)
    }

    #[test]
    fn default_link_links_and_case_folding() {
        let report = run("LINK links", LexicalConfig::default());
        assert!(report.complete);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].normalized_distance, 0.2);
        for text in [
            "link LINK",
            "Straße STRASSE",
            "café cafe\u{301}",
            "link link",
            "car automobile",
            "it its",
            "planet window",
        ] {
            assert!(
                run(text, LexicalConfig::default()).findings.is_empty(),
                "{text}"
            );
        }
    }
    #[test]
    fn utf8_spans_before_and_inside_both_locations() {
        let a = source("a.md", "😀 café");
        let b = source("b.md", "🦀 cafés");
        let report = analyze(&[b.clone(), a.clone()], &LexicalConfig::default());
        let evidence = &report.findings[0].evidence;
        assert_eq!(
            &a.content[evidence[0].span.start..evidence[0].span.end],
            "café"
        );
        assert_eq!(
            &b.content[evidence[1].span.start..evidence[1].span.end],
            "cafés"
        );
        assert_eq!(report, analyze(&[a, b], &LexicalConfig::default()));
    }
    #[test]
    fn configured_threshold_and_short_token_policy() {
        assert!(
            run(
                "link links",
                LexicalConfig {
                    threshold: 0.19,
                    ..Default::default()
                }
            )
            .findings
            .is_empty()
        );
        assert_eq!(
            run(
                "cat cats",
                LexicalConfig {
                    threshold: 0.25,
                    min_token_length: 3,
                    ..Default::default()
                }
            )
            .findings
            .len(),
            1
        );
        for threshold in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
            assert_eq!(
                run(
                    "link links",
                    LexicalConfig {
                        threshold,
                        ..Default::default()
                    }
                )
                .incomplete_reasons,
                [LexicalLimit::InvalidConfiguration]
            );
        }
    }
    #[test]
    fn bounds_are_observable_and_propagated() {
        let report = run(
            "link links linked",
            LexicalConfig {
                max_pairs: 1,
                ..Default::default()
            },
        );
        assert_eq!(report.pairs_examined, 1);
        assert_eq!(report.incomplete_reasons, [LexicalLimit::Pairs]);
        assert!(!report.findings[0].complete);
        for (config, reason) in [
            (
                LexicalConfig {
                    max_tokens: 1,
                    ..Default::default()
                },
                LexicalLimit::Tokens,
            ),
            (
                LexicalConfig {
                    max_input_bytes: 7,
                    ..Default::default()
                },
                LexicalLimit::InputBytes,
            ),
            (
                LexicalConfig {
                    max_token_length: 4,
                    ..Default::default()
                },
                LexicalLimit::TokenLength,
            ),
        ] {
            assert_eq!(run("link links", config).incomplete_reasons, [reason]);
        }
        assert!(
            run(
                "link",
                LexicalConfig {
                    max_tokens: 1,
                    max_input_bytes: 4,
                    ..Default::default()
                }
            )
            .complete
        );
        assert!(
            run(
                "link lin😀",
                LexicalConfig {
                    max_input_bytes: 10,
                    ..Default::default()
                }
            )
            .findings
            .is_empty()
        );
        let sources = [source("a", "link"), source("b", "links")];
        assert_eq!(
            analyze(
                &sources,
                &LexicalConfig {
                    max_sources: 1,
                    ..Default::default()
                }
            )
            .incomplete_reasons,
            [LexicalLimit::Sources]
        );
    }
    #[test]
    fn serialization_contains_no_source_and_scores_do_not_change() {
        let sources = vec![source("a.md", "PRIVATE_SENTINEL link links")];
        let config = HarnessLensConfig::default();
        let report = AnalysisEngine::new().analyze(".".into(), sources.clone(), vec![], &config);
        let json = serde_json::to_string(&report).unwrap();
        for secret in ["PRIVATE_SENTINEL", "\"link\"", "\"links\""] {
            assert!(!json.contains(secret));
        }
        let other = AnalysisEngine::new().analyze(
            ".".into(),
            sources,
            vec![],
            &HarnessLensConfig {
                lexical: LexicalConfig {
                    threshold: 0.0,
                    ..Default::default()
                },
                ..config
            },
        );
        assert_eq!(report.findings, other.findings);
        assert_eq!(report.scores, other.scores);
        assert_eq!(report.score_summary, other.score_summary);
    }
    #[test]
    fn bounded_algorithm_matches_reference_for_all_small_strings() {
        // Independent full matrix: no band, budget, early exit, or row reuse.
        fn reference_distance(left: &[char], right: &[char]) -> usize {
            let mut matrix = vec![vec![0; right.len() + 1]; left.len() + 1];
            for (row, cells) in matrix.iter_mut().enumerate() {
                cells[0] = row;
            }
            for (column, cell) in matrix[0].iter_mut().enumerate() {
                *cell = column;
            }
            for row in 1..=left.len() {
                for column in 1..=right.len() {
                    matrix[row][column] = (matrix[row - 1][column] + 1)
                        .min(matrix[row][column - 1] + 1)
                        .min(
                            matrix[row - 1][column - 1]
                                + usize::from(left[row - 1] != right[column - 1]),
                        );
                }
            }
            matrix[left.len()][right.len()]
        }

        let words: Vec<Vec<char>> = (0..=5)
            .flat_map(|length| {
                (0..1 << length).map(move |bits| {
                    (0..length)
                        .map(|i| if bits & (1 << i) == 0 { 'a' } else { 'b' })
                        .collect()
                })
            })
            .collect();
        for left in &words {
            for right in &words {
                let exact = reference_distance(left, right);
                for budget in 0..=5 {
                    assert_eq!(
                        bounded_distance(left, right, budget),
                        (exact <= budget).then_some(exact),
                        "left={left:?}, right={right:?}, budget={budget}"
                    );
                }
            }
        }
        assert_eq!(bounded_distance(&['a'; 64], &['z'; 64], 1), None);
    }
}
