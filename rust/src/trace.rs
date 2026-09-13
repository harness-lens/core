// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

//! Content-safe ordered action-trace contracts.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::path::{Component, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    EvidenceDescriptor, ObservationWindow, ObservedCost, RuntimeErrorClass,
    RuntimeObservationStatus, ScoreMethod, TextSpan,
};

/// Current provider-neutral action-trace schema version.
pub const ACTION_TRACE_SCHEMA_VERSION: u32 = 1;

/// Content-free reason why evidence is incomplete or was normalized.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CompletenessReason {
    /// Stable machine-readable reason code.
    pub code: String,
    /// Number of affected input records, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<usize>,
}

/// Explicit completeness state shared by trace and graph contracts.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceCompleteness {
    /// True only when no limiting or normalization condition was observed.
    pub complete: bool,
    /// Stable, content-free reasons for incomplete evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<CompletenessReason>,
}

impl Default for EvidenceCompleteness {
    fn default() -> Self {
        Self {
            complete: true,
            reasons: Vec::new(),
        }
    }
}

impl EvidenceCompleteness {
    pub(crate) fn canonicalize(&mut self) {
        self.reasons.sort();
        self.reasons.dedup();
    }

    pub(crate) fn validate(&self) -> Result<(), TraceValidationError> {
        if self.complete != self.reasons.is_empty() {
            return Err(TraceValidationError::InvalidCompleteness);
        }
        let mut previous = None;
        for reason in &self.reasons {
            validate_identifier("completeness reason", &reason.code)?;
            if reason.count == Some(0) {
                return Err(TraceValidationError::InvalidCompleteness);
            }
            if previous.is_some_and(|value: &CompletenessReason| value >= reason) {
                return Err(TraceValidationError::NonCanonicalOrder(
                    "completeness reasons",
                ));
            }
            previous = Some(reason);
        }
        Ok(())
    }
}

/// Safe source location attached to an observed action.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvidenceLocation {
    /// Path relative to the owning workspace root.
    pub path: PathBuf,
    /// One-based line number, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Exact UTF-8 byte range, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<TextSpan>,
}

impl EvidenceLocation {
    pub(crate) fn validate(&self) -> Result<(), TraceValidationError> {
        if self.path.as_os_str().is_empty()
            || self.path.is_absolute()
            || self.path.components().any(|part| {
                matches!(
                    part,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(TraceValidationError::InvalidLocation);
        }
        if self.line == Some(0) || self.span.is_some_and(|span| span.start > span.end) {
            return Err(TraceValidationError::InvalidLocation);
        }
        Ok(())
    }
}

/// Safe identity for one observed action or tool.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ActionIdentity {
    /// Stable logical identity used to aggregate repeated actions.
    pub id: String,
    /// Content-safe display label. It must not contain arguments or output.
    pub label: String,
    /// Stable category used by filters.
    pub category: String,
}

/// Provider-neutral token usage attributed to one observed turn.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObservedTokenUsage {
    /// Prompt or input tokens, when supplied by the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    /// Completion or output tokens, when supplied by the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    /// Cached input tokens, when supplied; this is a subset of input tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_input_tokens: Option<u64>,
    /// Total tokens charged to the turn before any cache discount.
    pub total_tokens: u64,
    /// Whether any token count was estimated instead of measured by the source.
    pub estimated: bool,
}

impl ObservedTokenUsage {
    fn validate(&self) -> Result<(), TraceValidationError> {
        if self
            .cached_input_tokens
            .is_some_and(|cached| self.input_tokens.is_none_or(|input| cached > input))
        {
            return Err(TraceValidationError::InvalidMetric);
        }
        match (self.input_tokens, self.output_tokens) {
            (Some(input), Some(output)) if input.checked_add(output) != Some(self.total_tokens) => {
                Err(TraceValidationError::InvalidMetric)
            }
            (Some(input), None) if input > self.total_tokens => {
                Err(TraceValidationError::InvalidMetric)
            }
            (None, Some(output)) if output > self.total_tokens => {
                Err(TraceValidationError::InvalidMetric)
            }
            _ => Ok(()),
        }
    }
}

/// One ordered action containing no prompt, arguments, output, transcript, or stderr.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionObservation {
    /// Stable observation identity.
    pub id: String,
    /// Stable session identity.
    pub session_id: String,
    /// Provider-supplied order within the session.
    pub sequence: u64,
    /// Normalized timestamp, when supplied by the adapter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_at: Option<String>,
    /// Safe logical action identity.
    pub action: ActionIdentity,
    /// Sanitized terminal status.
    pub status: RuntimeObservationStatus,
    /// Wall-clock duration in microseconds; missing never means zero.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_micros: Option<u64>,
    /// Retry count; missing never means zero.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<u32>,
    /// Optional observed cost, separate from static context estimates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<ObservedCost>,
    /// Optional measured or explicitly estimated token usage for this turn.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<ObservedTokenUsage>,
    /// Stable error class; absent for successful observations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_class: Option<RuntimeErrorClass>,
    /// Safe model identity, when attributable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Safe skill, configuration, or harness identity, when attributable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_identity: Option<String>,
    /// Immutable revision, when attributable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    /// Optional safe source navigation location.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<EvidenceLocation>,
    /// Declared evidence method and assumptions.
    pub evidence: EvidenceDescriptor,
}

impl ActionObservation {
    /// Validates content-safe identity and evidence invariants.
    pub fn validate(&self) -> Result<(), TraceValidationError> {
        validate_identifier("observation id", &self.id)?;
        validate_identifier("session id", &self.session_id)?;
        validate_identifier("action id", &self.action.id)?;
        validate_label("action label", &self.action.label)?;
        validate_identifier("action category", &self.action.category)?;
        if let Some(value) = &self.observed_at {
            validate_opaque("observation timestamp", value)?;
        }
        for (field, value) in [
            ("model identity", self.model.as_deref()),
            ("asset identity", self.asset_identity.as_deref()),
            ("revision", self.revision.as_deref()),
        ] {
            if let Some(value) = value {
                validate_identifier(field, value)?;
            }
        }
        if self.status == RuntimeObservationStatus::Success && self.error_class.is_some() {
            return Err(TraceValidationError::InvalidStatus);
        }
        if let Some(cost) = &self.cost {
            if !cost.value.is_finite() || cost.value < 0.0 {
                return Err(TraceValidationError::InvalidMetric);
            }
            validate_identifier("cost unit", &cost.unit)?;
        }
        if let Some(token_usage) = &self.token_usage {
            token_usage.validate()?;
        }
        if let Some(location) = &self.location {
            location.validate()?;
        }
        validate_evidence(&self.evidence)
    }
}

/// Versioned, bounded sequence of sanitized observed actions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionTrace {
    /// Trace schema version.
    pub schema_version: u32,
    /// Inclusive observation window represented by this trace.
    pub window: ObservationWindow,
    /// Whether all requested source observations were usable.
    pub completeness: EvidenceCompleteness,
    /// Canonically ordered, sanitized observations.
    #[serde(default)]
    pub observations: Vec<ActionObservation>,
    /// Total source observations before bounds, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_observations: Option<usize>,
    /// Opaque safe continuation token, when more observations exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl ActionTrace {
    /// Sorts all set-like fields and observations into canonical order.
    #[must_use]
    pub fn canonicalize(mut self) -> Self {
        self.completeness.canonicalize();
        self.observations.sort_by(|left, right| {
            (&left.session_id, left.sequence, &left.id).cmp(&(
                &right.session_id,
                right.sequence,
                &right.id,
            ))
        });
        self
    }

    /// Validates schema, bounds, ordering, uniqueness, and safe fields.
    pub fn validate(&self, max_observations: usize) -> Result<(), TraceValidationError> {
        if self.schema_version != ACTION_TRACE_SCHEMA_VERSION {
            return Err(TraceValidationError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        if max_observations == 0 || self.observations.len() > max_observations {
            return Err(TraceValidationError::BoundExceeded {
                kind: "observations",
                limit: max_observations,
                actual: self.observations.len(),
            });
        }
        validate_window(&self.window)?;
        self.completeness.validate()?;
        if self
            .total_observations
            .is_some_and(|total| total < self.observations.len())
        {
            return Err(TraceValidationError::InvalidTotal);
        }
        if let Some(cursor) = &self.next_cursor {
            validate_identifier("continuation cursor", cursor)?;
        }

        let mut ids = BTreeSet::new();
        let mut positions = BTreeSet::new();
        let mut previous: Option<(&str, u64, &str)> = None;
        for observation in &self.observations {
            observation.validate()?;
            if !ids.insert(observation.id.as_str()) {
                return Err(TraceValidationError::DuplicateObservationId);
            }
            if !positions.insert((observation.session_id.as_str(), observation.sequence)) {
                return Err(TraceValidationError::DuplicateSequence);
            }
            let key = (
                observation.session_id.as_str(),
                observation.sequence,
                observation.id.as_str(),
            );
            if previous.is_some_and(|value| value >= key) {
                return Err(TraceValidationError::NonCanonicalOrder("observations"));
            }
            previous = Some(key);
        }
        Ok(())
    }
}

/// Invalid action-trace or shared evidence contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraceValidationError {
    /// Schema version is unsupported.
    UnsupportedSchemaVersion(u32),
    /// A bounded collection exceeded its declared limit.
    BoundExceeded {
        /// Collection name.
        kind: &'static str,
        /// Declared limit.
        limit: usize,
        /// Actual collection length.
        actual: usize,
    },
    /// Stable identifier is empty, too long, or contains unsafe characters.
    InvalidIdentifier(&'static str),
    /// Display label is empty, too long, or contains control characters.
    InvalidLabel(&'static str),
    /// Observation window is invalid.
    InvalidWindow,
    /// Completeness flag and reasons disagree.
    InvalidCompleteness,
    /// Source location is absolute, escaping, or internally inconsistent.
    InvalidLocation,
    /// Status and error fields disagree.
    InvalidStatus,
    /// Metric is non-finite, negative, or missing required evidence.
    InvalidMetric,
    /// Evidence metadata does not match its method.
    InvalidEvidence,
    /// Total count is smaller than visible records.
    InvalidTotal,
    /// Observation identity appears more than once.
    DuplicateObservationId,
    /// Session sequence position contains more than one action.
    DuplicateSequence,
    /// Serialized collection is not in canonical order.
    NonCanonicalOrder(&'static str),
}

impl fmt::Display for TraceValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(version) => {
                write!(
                    formatter,
                    "unsupported action-trace schema version: {version}"
                )
            }
            Self::BoundExceeded {
                kind,
                limit,
                actual,
            } => write!(formatter, "{kind} count {actual} exceeds limit {limit}"),
            Self::InvalidIdentifier(field) => write!(formatter, "invalid {field}"),
            Self::InvalidLabel(field) => write!(formatter, "invalid {field}"),
            Self::InvalidWindow => formatter.write_str("invalid observation window"),
            Self::InvalidCompleteness => formatter.write_str("invalid completeness state"),
            Self::InvalidLocation => formatter.write_str("invalid evidence location"),
            Self::InvalidStatus => {
                formatter.write_str("observation status contradicts error class")
            }
            Self::InvalidMetric => formatter.write_str("invalid observed metric"),
            Self::InvalidEvidence => formatter.write_str("invalid evidence descriptor"),
            Self::InvalidTotal => {
                formatter.write_str("total observations is smaller than visible observations")
            }
            Self::DuplicateObservationId => formatter.write_str("duplicate observation identity"),
            Self::DuplicateSequence => formatter.write_str("duplicate session sequence"),
            Self::NonCanonicalOrder(kind) => {
                write!(formatter, "{kind} are not canonically ordered")
            }
        }
    }
}

impl Error for TraceValidationError {}

pub(crate) fn validate_identifier(
    field: &'static str,
    value: &str,
) -> Result<(), TraceValidationError> {
    if value.is_empty()
        || value.len() > 128
        || !value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'/' | b'@')
        })
    {
        return Err(TraceValidationError::InvalidIdentifier(field));
    }
    Ok(())
}

pub(crate) fn validate_label(field: &'static str, value: &str) -> Result<(), TraceValidationError> {
    if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return Err(TraceValidationError::InvalidLabel(field));
    }
    Ok(())
}

pub(crate) fn validate_opaque(
    field: &'static str,
    value: &str,
) -> Result<(), TraceValidationError> {
    if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        return Err(TraceValidationError::InvalidIdentifier(field));
    }
    Ok(())
}

pub(crate) fn validate_window(window: &ObservationWindow) -> Result<(), TraceValidationError> {
    validate_opaque("observation window start", &window.start)?;
    validate_opaque("observation window end", &window.end)?;
    if window.start > window.end {
        return Err(TraceValidationError::InvalidWindow);
    }
    Ok(())
}

pub(crate) fn validate_evidence(evidence: &EvidenceDescriptor) -> Result<(), TraceValidationError> {
    match evidence.method {
        ScoreMethod::Deterministic => {
            if evidence.sample_size.is_some() || evidence.uncertainty.is_some() {
                return Err(TraceValidationError::InvalidEvidence);
            }
        }
        ScoreMethod::Heuristic => {
            if evidence.assumptions.is_empty() || evidence.uncertainty.is_some() {
                return Err(TraceValidationError::InvalidEvidence);
            }
        }
        ScoreMethod::Statistical => {
            if evidence.sample_size == Some(0) || evidence.sample_size.is_none() {
                return Err(TraceValidationError::InvalidEvidence);
            }
        }
        ScoreMethod::Probabilistic => {
            let Some(interval) = &evidence.uncertainty else {
                return Err(TraceValidationError::InvalidEvidence);
            };
            if !interval.lower.is_finite()
                || !interval.upper.is_finite()
                || !interval.level.is_finite()
                || !(0.0..=1.0).contains(&interval.lower)
                || !(0.0..=1.0).contains(&interval.upper)
                || !(0.0..=1.0).contains(&interval.level)
                || interval.lower > interval.upper
                || interval.method.is_empty()
            {
                return Err(TraceValidationError::InvalidEvidence);
            }
        }
    }
    if evidence
        .assumptions
        .iter()
        .any(|value| value.is_empty() || value.len() > 256 || value.chars().any(char::is_control))
    {
        return Err(TraceValidationError::InvalidEvidence);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(id: &str, sequence: u64) -> ActionObservation {
        ActionObservation {
            id: id.to_owned(),
            session_id: "session-1".to_owned(),
            sequence,
            observed_at: Some(format!("2026-09-13T00:00:0{sequence}Z")),
            action: ActionIdentity {
                id: "tool.read".to_owned(),
                label: "Read".to_owned(),
                category: "tool".to_owned(),
            },
            status: RuntimeObservationStatus::Success,
            duration_micros: Some(10),
            retry_count: Some(0),
            cost: None,
            token_usage: Some(ObservedTokenUsage {
                input_tokens: Some(80),
                output_tokens: Some(40),
                cached_input_tokens: Some(20),
                total_tokens: 120,
                estimated: false,
            }),
            error_class: None,
            model: None,
            asset_identity: Some("AGENTS.md".to_owned()),
            revision: Some("74072145".to_owned()),
            location: Some(EvidenceLocation {
                path: PathBuf::from("AGENTS.md"),
                line: Some(1),
                span: Some(TextSpan { start: 0, end: 4 }),
            }),
            evidence: EvidenceDescriptor::default(),
        }
    }

    #[test]
    fn action_trace_round_trip_is_canonical_and_content_safe() {
        let trace = ActionTrace {
            schema_version: ACTION_TRACE_SCHEMA_VERSION,
            window: ObservationWindow {
                start: "2026-09-13T00:00:00Z".to_owned(),
                end: "2026-09-13T01:00:00Z".to_owned(),
            },
            completeness: EvidenceCompleteness::default(),
            observations: vec![observation("obs-2", 2), observation("obs-1", 1)],
            total_observations: Some(2),
            next_cursor: None,
        }
        .canonicalize();

        trace.validate(10).unwrap();
        let encoded = serde_json::to_string(&trace).unwrap();
        let decoded: ActionTrace = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, trace);
        assert_eq!(decoded.observations[0].id, "obs-1");
        assert_eq!(
            decoded.observations[0].token_usage.unwrap().total_tokens,
            120
        );
        for forbidden in [
            "\"arguments\"",
            "\"output\"",
            "\"prompt\"",
            "\"stderr\"",
            "\"transcript\"",
        ] {
            assert!(!encoded.contains(forbidden));
        }
    }

    #[test]
    fn trace_rejects_duplicate_positions_and_unsafe_locations() {
        let mut second = observation("obs-2", 1);
        second.location = Some(EvidenceLocation {
            path: PathBuf::from("../secret"),
            line: None,
            span: None,
        });
        let trace = ActionTrace {
            schema_version: ACTION_TRACE_SCHEMA_VERSION,
            window: ObservationWindow {
                start: "a".to_owned(),
                end: "z".to_owned(),
            },
            completeness: EvidenceCompleteness::default(),
            observations: vec![observation("obs-1", 1), second],
            total_observations: Some(2),
            next_cursor: None,
        }
        .canonicalize();

        assert_eq!(
            trace.validate(10),
            Err(TraceValidationError::InvalidLocation)
        );
        assert_eq!(
            trace.observations[1].validate(),
            Err(TraceValidationError::InvalidLocation)
        );
        let mut trace = trace;
        trace.observations[1].location = None;
        assert_eq!(
            trace.validate(10),
            Err(TraceValidationError::DuplicateSequence)
        );
    }

    #[test]
    fn statistical_and_probabilistic_evidence_require_uncertainty_metadata() {
        let mut value = observation("obs-1", 1);
        value.evidence.method = ScoreMethod::Statistical;
        assert_eq!(value.validate(), Err(TraceValidationError::InvalidEvidence));
        value.evidence.sample_size = Some(1);
        assert!(value.validate().is_ok());

        value.evidence.method = ScoreMethod::Probabilistic;
        assert_eq!(value.validate(), Err(TraceValidationError::InvalidEvidence));
    }

    #[test]
    fn token_usage_requires_consistent_totals_and_cache_counts() {
        let mut value = observation("obs-1", 1);
        value.token_usage = Some(ObservedTokenUsage {
            input_tokens: Some(80),
            output_tokens: Some(41),
            cached_input_tokens: Some(20),
            total_tokens: 120,
            estimated: false,
        });
        assert_eq!(value.validate(), Err(TraceValidationError::InvalidMetric));

        value.token_usage = Some(ObservedTokenUsage {
            input_tokens: Some(80),
            output_tokens: Some(40),
            cached_input_tokens: Some(81),
            total_tokens: 120,
            estimated: false,
        });
        assert_eq!(value.validate(), Err(TraceValidationError::InvalidMetric));
    }
}
