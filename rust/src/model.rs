// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A kind of harness source that can affect an agent workspace.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum HarnessSourceKind {
    /// Human- or project-authored instructions.
    Instructions,
    /// Provider or project rules.
    Rules,
    /// Discoverable agent skills.
    Skills,
    /// Lifecycle or tool hooks.
    Hooks,
    /// Agent declarations.
    Agents,
    /// Runtime or provider configuration.
    Configuration,
    /// Persistent memory sources.
    Memory,
    /// Declared workflows.
    Workflows,
    /// Source recognized by custom configuration or a plugin.
    Other,
}

/// Loaded input supplied to the deterministic core.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HarnessSource {
    /// Path relative to the scanned root.
    pub path: PathBuf,
    /// Semantic source category.
    pub kind: HarnessSourceKind,
    /// Directory where this source takes effect.
    pub scope: PathBuf,
    /// UTF-8 source content. Reports never serialize this field directly.
    pub content: String,
}

impl HarnessSource {
    /// Produces the content-free representation used in public reports.
    #[must_use]
    pub fn record(&self) -> SourceRecord {
        let characters = self.content.chars().count();
        SourceRecord {
            path: self.path.clone(),
            kind: self.kind,
            scope: self.scope.clone(),
            bytes: self.content.len(),
            characters,
            lines: self.content.lines().count(),
            inclusion_depth: 0,
            estimated_tokens: TokenEstimate {
                value: characters.div_ceil(4),
                tokenizer: "unicode_scalar_div_4".to_owned(),
                basis: "Unicode scalar count divided by four".to_owned(),
                method: ScoreMethod::Heuristic,
            },
            configured_input_cost: None,
            findings_count: 0,
            provenance: vec![ProvenanceLink {
                relationship: ProvenanceRelationship::DirectDiscovery,
                path: self.path.clone(),
                span: None,
                method: ScoreMethod::Deterministic,
            }],
        }
    }
}

/// Content-free source metadata safe for reports and integrations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceRecord {
    /// Path relative to the scanned root.
    pub path: PathBuf,
    /// Semantic source category.
    pub kind: HarnessSourceKind,
    /// Directory where this source takes effect.
    pub scope: PathBuf,
    /// Loaded UTF-8 byte count.
    pub bytes: usize,
    /// Unicode scalar count.
    #[serde(default)]
    pub characters: usize,
    /// Logical line count.
    #[serde(default)]
    pub lines: usize,
    /// Shortest known inclusion depth from direct workspace discovery.
    #[serde(default)]
    pub inclusion_depth: usize,
    /// Explicitly labeled token estimate.
    #[serde(default)]
    pub estimated_tokens: TokenEstimate,
    /// Configured static input cost, separate from observed runtime cost.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configured_input_cost: Option<ConfiguredInputCost>,
    /// Non-pass findings attributed to this file.
    #[serde(default)]
    pub findings_count: usize,
    /// Safe content-free paths explaining discovery or inclusion.
    #[serde(default)]
    pub provenance: Vec<ProvenanceLink>,
}

/// Explainable token estimate for one source.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TokenEstimate {
    /// Estimated token count.
    pub value: usize,
    /// Stable estimator or tokenizer identifier.
    pub tokenizer: String,
    /// Human-readable estimator basis.
    pub basis: String,
    /// Method classification; estimates are normally heuristic.
    pub method: ScoreMethod,
}

impl Default for TokenEstimate {
    fn default() -> Self {
        Self {
            value: 0,
            tokenizer: "unknown".to_owned(),
            basis: "not supplied".to_owned(),
            method: ScoreMethod::Heuristic,
        }
    }
}

/// Configured static context cost for one harness invocation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfiguredInputCost {
    /// Cost amount per invocation.
    pub value: f64,
    /// Unit including currency, such as `USD/invocation`.
    pub unit: String,
    /// Caller-supplied pricing reference, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    /// Method classification inherited from the token estimate and formula.
    pub method: ScoreMethod,
}

/// How a source entered the bounded analysis graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceRelationship {
    /// Source matched deterministic workspace discovery.
    DirectDiscovery,
    /// Another source explicitly referenced this source.
    Reference,
    /// Source was discovered as a skill dependency.
    SkillDependency,
}

/// Content-free provenance location.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceLink {
    /// Relationship represented by this link.
    pub relationship: ProvenanceRelationship,
    /// Source or target path relative to the scan root.
    pub path: PathBuf,
    /// UTF-8 byte range containing the reference, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<TextSpan>,
    /// Method used to identify this link.
    pub method: ScoreMethod,
}

/// Resolution state for one bounded inclusion edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InclusionStatus {
    /// Target resolved inside the scan root.
    Resolved,
    /// Referenced target does not exist.
    Missing,
    /// Edge closes an inclusion cycle.
    Cycle,
    /// Target matched an explicit ignore rule.
    Ignored,
    /// Target leaves the scan root.
    OutOfRoot,
    /// Target could not be inspected safely.
    Unavailable,
}

/// Content-free directed edge in the harness inclusion graph.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InclusionEdge {
    /// Referring source; absent for direct workspace discovery.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PathBuf>,
    /// Referenced or discovered target path.
    pub target: PathBuf,
    /// Inclusion depth assigned by bounded traversal.
    pub depth: usize,
    /// Observable resolution state.
    pub status: InclusionStatus,
    /// UTF-8 byte range containing the reference, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<TextSpan>,
    /// Method used to identify this edge.
    pub method: ScoreMethod,
    /// Declared assumptions for heuristic edges.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assumptions: Vec<String>,
}

/// Severity attached to an evidence-bearing finding.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Rule completed successfully.
    Pass,
    /// Informational result.
    Info,
    /// Condition deserves attention but does not invalidate the scan.
    Warning,
    /// Condition prevents a trustworthy result.
    Error,
}

/// Half-open UTF-8 byte range within a [`HarnessSource`].
///
/// Byte offsets keep the core independent from editor protocols. Presentation
/// adapters are responsible for converting them to their native position
/// encoding, such as the UTF-16 positions used by LSP clients.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TextSpan {
    /// Inclusive starting byte offset.
    pub start: usize,
    /// Exclusive ending byte offset.
    pub end: usize,
}

/// A second source location related to an evidence-bearing finding.
///
/// The primary location remains on [`Finding`]. This small relation keeps
/// duplicate and conflict diagnostics navigable without copying source text
/// into the report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FindingLocation {
    /// Related source path.
    pub path: PathBuf,
    /// One-based line number, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Exact UTF-8 byte range, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<TextSpan>,
}

/// Deterministic evidence emitted by the core or a plugin.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    /// Severity of the finding.
    pub severity: Severity,
    /// Stable rule identifier.
    pub rule_id: String,
    /// Human-readable result.
    pub message: String,
    /// Related source path, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    /// One-based line number, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    /// Exact UTF-8 byte range in the related source, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<TextSpan>,
    /// Minimal supporting evidence, when safe to expose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    /// Other source locations needed to understand this finding.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<FindingLocation>,
    /// Core or plugin identifier that produced this finding.
    pub source: String,
}

/// Named raw measurement produced by deterministic analysis.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Metric {
    /// Stable metric name.
    pub name: String,
    /// Numeric value in its natural unit.
    pub value: f64,
    /// Optional unit or profile identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Source path for a per-file measurement, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    /// Caller-supplied pricing or evaluation reference, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    /// Core or plugin identifier that produced this metric.
    pub source: String,
}

/// Whether a normalized score may participate in quality aggregation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreCategory {
    /// Static quality signal eligible for a quality summary.
    Quality,
    /// Safety constraint reported separately and never averaged into quality.
    Safety,
    /// Repeatability or robustness signal.
    Reliability,
    /// Cost, latency, or resource signal.
    Performance,
}

/// How a normalized score was produced.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreMethod {
    /// Exact rule with no estimated uncertainty.
    #[default]
    Deterministic,
    /// Explainable rule-of-thumb with explicit evidence.
    Heuristic,
    /// Aggregate computed from observed samples.
    Statistical,
    /// Deterministic probability estimate computed from declared assumptions.
    Probabilistic,
}

/// Uncertainty attached to a statistical or probabilistic score.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceEstimate {
    /// Lower bound in the normalized interval.
    pub lower: f64,
    /// Upper bound in the normalized interval.
    pub upper: f64,
    /// Confidence level, such as `0.95`.
    pub level: f64,
    /// Estimator name and assumptions.
    pub method: String,
}

/// Invalid normalized score construction.
#[derive(Clone, Debug, PartialEq)]
pub enum ScoreError {
    /// Value or threshold was not finite.
    NotFinite,
    /// Value or threshold was outside the normalized interval.
    OutsideUnitInterval(f64),
}

impl fmt::Display for ScoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFinite => formatter.write_str("score value and threshold must be finite"),
            Self::OutsideUnitInterval(value) => {
                write!(
                    formatter,
                    "score value must be between 0.0 and 1.0: {value}"
                )
            }
        }
    }
}

impl Error for ScoreError {}

/// Normalized, evidence-bearing score with derived pass/fail state.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Score {
    /// Stable score identifier.
    pub id: String,
    /// Aggregation and policy category.
    pub category: ScoreCategory,
    /// Computation method.
    pub method: ScoreMethod,
    /// Normalized value from 0.0 to 1.0.
    pub value: f64,
    /// Configured pass threshold from 0.0 to 1.0.
    pub threshold: f64,
    /// Derived state. Callers cannot choose it independently.
    pub passed: bool,
    /// Number of observations behind this result, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_size: Option<usize>,
    /// Optional uncertainty estimate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<ConfidenceEstimate>,
    /// Human-readable interpretation and assumptions.
    pub reason: String,
    /// Structured, non-secret evidence.
    pub evidence: BTreeMap<String, String>,
    /// Core or plugin identifier that produced this score.
    pub source: String,
}

impl Score {
    /// Constructs a valid normalized score and derives `passed`.
    pub fn new(
        id: impl Into<String>,
        category: ScoreCategory,
        method: ScoreMethod,
        value: f64,
        threshold: f64,
        reason: impl Into<String>,
        source: impl Into<String>,
    ) -> Result<Self, ScoreError> {
        if !value.is_finite() || !threshold.is_finite() {
            return Err(ScoreError::NotFinite);
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(ScoreError::OutsideUnitInterval(value));
        }
        if !(0.0..=1.0).contains(&threshold) {
            return Err(ScoreError::OutsideUnitInterval(threshold));
        }

        Ok(Self {
            id: id.into(),
            category,
            method,
            value,
            threshold,
            passed: value >= threshold,
            sample_size: None,
            confidence: None,
            reason: reason.into(),
            evidence: BTreeMap::new(),
            source: source.into(),
        })
    }
}

/// Observable status for one plugin execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginExecutionStatus {
    /// Plugin completed and its output was accepted.
    Completed,
    /// Plugin was disabled by configuration.
    Disabled,
    /// Configured plugin was not registered by the host.
    Unavailable,
    /// Plugin returned an error.
    Failed,
}

/// Execution trace included in every analysis report.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginExecution {
    /// Stable plugin identifier.
    pub id: String,
    /// Terminal execution state.
    pub status: PluginExecutionStatus,
    /// Wall-clock execution time measured by the host.
    pub duration_micros: u64,
    /// Optional failure or availability detail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Category-aware report aggregation.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ScoreSummary {
    /// Mean of quality-category scores, when any exist.
    pub quality_mean: Option<f64>,
    /// Failed safety constraints. Safety is never included in `quality_mean`.
    pub safety_violations: usize,
    /// Mean score per category.
    pub by_category: BTreeMap<ScoreCategory, f64>,
}

/// Runtime collection mode reported by an adapter.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMode {
    /// Runtime collection is disabled.
    #[default]
    Off,
    /// Adapter captured aggregate evidence from an opt-in live source.
    Live,
    /// Adapter read a canonical aggregate snapshot without process launch.
    Snapshot,
}

/// Availability of optional runtime evidence.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeAvailability {
    /// Runtime evidence is disabled.
    #[default]
    Off,
    /// Requested runtime evidence is complete for its declared window.
    Ready,
    /// Some bounded runtime evidence is available and limitations are visible.
    Partial,
    /// Collection failed without invalidating deterministic analysis.
    Failed,
}

/// Sanitized terminal state of one observed tool call.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeObservationStatus {
    /// Tool call completed successfully.
    Success,
    /// Tool call returned an error.
    Error,
    /// Tool call exceeded a declared time bound.
    Timeout,
    /// Tool call was cancelled.
    Cancelled,
}

/// Stable runtime error class that cannot contain raw stderr or output.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeErrorClass {
    /// Executable, snapshot, or integration was unavailable.
    Unavailable,
    /// Operation exceeded its time bound.
    Timeout,
    /// Access was denied.
    PermissionDenied,
    /// Aggregate or observation data was invalid.
    InvalidData,
    /// Network operation failed.
    Network,
    /// Tool returned a non-success result without safe detail.
    ToolFailure,
    /// Failure could not be classified safely.
    Unknown,
}

/// Inclusive runtime comparison window identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ObservationWindow {
    /// Stable window identifier or normalized start timestamp.
    pub start: String,
    /// Stable window identifier or normalized end timestamp.
    pub end: String,
}

/// Sanitized cost attributed to one runtime observation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObservedCost {
    /// Non-negative cost amount.
    pub value: f64,
    /// Currency or billing unit.
    pub unit: String,
    /// Whether token or pricing inputs were estimated.
    pub estimated: bool,
}

/// Evidence declaration shared by runtime and effectiveness records.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EvidenceDescriptor {
    /// Method used to produce the record.
    pub method: ScoreMethod,
    /// Safe assumptions required by heuristic or probabilistic methods.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assumptions: Vec<String>,
    /// Number of observations behind a statistical result.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_size: Option<usize>,
    /// Prior and interval method for probabilistic results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncertainty: Option<ConfidenceEstimate>,
}

/// One bounded observation containing no raw arguments, output, transcript, or stderr.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeObservation {
    /// Stable tool identity.
    pub tool: String,
    /// Safe category used for aggregate filters.
    pub category: String,
    /// Sanitized terminal status.
    pub status: RuntimeObservationStatus,
    /// Wall-clock duration in microseconds.
    pub duration_micros: u64,
    /// Number of retries attributed to this observation.
    pub retry_count: u32,
    /// Optional observed cost, separate from static context estimates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<ObservedCost>,
    /// Stable error class; absent for successful observations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_class: Option<RuntimeErrorClass>,
    /// Safe model identity, when attributable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Safe skill, configuration, or harness identity, when attributable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_identity: Option<String>,
    /// Immutable revision required for attributed comparisons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,
    /// Observation time window.
    pub window: ObservationWindow,
    /// Declared evidence method and assumptions.
    pub evidence: EvidenceDescriptor,
}

/// Observable optional runtime section of a deterministic report.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RuntimeReport {
    /// Consent-controlled collection mode.
    pub mode: RuntimeMode,
    /// Runtime evidence availability.
    pub availability: RuntimeAvailability,
    /// Sanitized bounded observations.
    #[serde(default)]
    pub observations: Vec<RuntimeObservation>,
    /// Total observations before pagination or bounding.
    pub total_observations: usize,
    /// Opaque safe continuation token, when more observations exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    /// Stable failure classes observed while collecting optional evidence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<RuntimeErrorClass>,
}

/// Conservative direction for an attributed before/after comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectivenessState {
    /// Identity, revision, window, sample, or uncertainty was insufficient.
    InsufficientEvidence,
    /// Declared thresholds classify the result as improving.
    Improving,
    /// Declared thresholds classify the result as stable.
    Stable,
    /// Declared thresholds classify the result as degrading.
    Degrading,
}

/// Attributed before/after assessment without a fabricated normalized file score.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EffectivenessAssessment {
    /// Safe file, skill, or configuration identity.
    pub asset_identity: String,
    /// Immutable current revision.
    pub revision: String,
    /// Baseline observation window.
    pub baseline: ObservationWindow,
    /// Current observation window.
    pub current: ObservationWindow,
    /// Conservative comparison result.
    pub state: EffectivenessState,
    /// Named metric deltas with documented units supplied by the adapter.
    pub deltas: BTreeMap<String, f64>,
    /// Declared comparison method, samples, uncertainty, and assumptions.
    pub evidence: EvidenceDescriptor,
}

/// Complete provider-neutral analysis result.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnalysisReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Scanned workspace root.
    pub root: PathBuf,
    /// Whether discovery and source loading covered the requested workspace.
    #[serde(default)]
    pub completeness: ScanCompleteness,
    /// Content-free discovered source records.
    pub sources: Vec<SourceRecord>,
    /// Bounded, content-free discovery and reference graph.
    #[serde(default)]
    pub inclusions: Vec<InclusionEdge>,
    /// Evidence-bearing findings.
    pub findings: Vec<Finding>,
    /// Raw deterministic measurements.
    pub metrics: Vec<Metric>,
    /// Normalized scores.
    pub scores: Vec<Score>,
    /// Category-aware aggregation.
    pub score_summary: ScoreSummary,
    /// Plugin execution trace for observability.
    pub plugin_executions: Vec<PluginExecution>,
    /// Optional sanitized runtime evidence; defaults to disabled.
    #[serde(default)]
    pub runtime: RuntimeReport,
    /// Attributed comparisons; empty means no sufficient runtime evidence.
    #[serde(default)]
    pub effectiveness: Vec<EffectivenessAssessment>,
    /// Lexical heuristic evidence, never included in deterministic scores.
    #[serde(default)]
    pub lexical: crate::lexical::LexicalReport,
}

/// Whether the adapter could inspect every relevant path it encountered.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScanCompleteness {
    /// `true` only when no limiting condition was observed.
    pub complete: bool,
    /// Stable, content-free reasons why a scan is incomplete.
    pub reasons: Vec<IncompleteReason>,
}

impl Default for ScanCompleteness {
    fn default() -> Self {
        Self {
            complete: true,
            reasons: Vec::new(),
        }
    }
}

/// One content-free reason that prevents an authoritative workspace scan.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IncompleteReason {
    /// Stable machine-readable reason code.
    pub code: String,
    /// Related path, relative to the scan root when possible.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

impl AnalysisReport {
    /// Returns aggregate report counts.
    #[must_use]
    pub fn summary(&self) -> ScanSummary {
        ScanSummary {
            sources: self.sources.len(),
            diagnostics: self
                .findings
                .iter()
                .filter(|finding| matches!(finding.severity, Severity::Warning | Severity::Error))
                .count(),
        }
    }
}

/// Aggregate, content-free result of a workspace scan.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScanSummary {
    /// Number of recognized harness sources.
    pub sources: usize,
    /// Number of warning and error findings.
    pub diagnostics: usize,
}

impl ScanSummary {
    /// Returns whether the scan found no harness sources or diagnostics.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.sources == 0 && self.diagnostics == 0
    }
}
