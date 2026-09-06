// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

//! Provider-neutral, content-free contracts. Metadata is supplied by compiled-in
//! adapters, never used as executable instructions. No provider can replace Native.

use crate::{ScoreMethod, TextSpan};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Stable Native provider identity.
pub const NATIVE_ID: &str = "harness-lens-native";
/// Stable optional CodeBurn identity.
pub const CODEBURN_ID: &str = "codeburn";
/// Maximum catalog or merge providers.
pub const MAX_PROVIDERS: usize = 16;
/// Maximum contributions per provider.
pub const MAX_CONTRIBUTIONS: usize = 2048;

/// Compiled-in provider description; contains no execution instructions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderDescriptor {
    /// Stable ID; catalog adapters own the namespace.
    pub id: String,
    /// Human-facing name.
    pub display_name: String,
    /// Accepted installed version, absent until detected.
    pub version: Option<String>,
    /// SPDX license expression, absent if not yet verified.
    pub license: Option<String>,
    /// Upstream source repository URL, supplied locally.
    pub source_url: String,
    /// Supported adapter capabilities.
    pub capabilities: Vec<ProviderCapability>,
    /// Required local settings, with no setting values.
    pub configuration: Vec<ConfigurationRequirement>,
    /// Supported operating systems, as established by the adapter.
    pub platforms: Vec<ProviderPlatform>,
    /// Methods this provider can contribute.
    pub methods: Vec<ScoreMethod>,
    /// False for Native; adapters must also enforce this invariant.
    pub optional: bool,
}

/// Provider capability, independent of editor presentation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCapability {
    /// Deterministic source analysis.
    DeterministicAnalysis,
    /// Lexical heuristic candidates.
    LexicalSimilarity,
    /// Aggregate runtime evidence.
    RuntimeAggregates,
    /// Canonical local snapshots.
    Snapshots,
    /// Adapter can produce an installation plan.
    InstallationPlanning,
}

/// Required local input; no secrets or paths are included.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigurationRequirement {
    /// Explicit executable location.
    Executable,
    /// Explicit off/live/snapshot mode.
    RuntimeMode,
    /// Canonical snapshot location.
    SnapshotPath,
}

/// Supported local operating system.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderPlatform {
    /// Linux host.
    Linux,
    /// macOS host.
    Macos,
    /// Windows host.
    Windows,
}

/// Safe error classes; never include process errors, arguments, or output.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderError {
    /// Unknown or forbidden provider identity.
    InvalidProvider,
    /// Executable does not exist.
    NotFound,
    /// Version response is invalid or unsupported.
    InvalidVersion,
    /// Local configuration cannot be used safely.
    ConfigurationError,
    /// Process launch or execution failed.
    RuntimeFailure,
    /// Workspace trust or virtual filesystem policy blocks access.
    WorkspaceBlocked,
    /// Process did not finish within its deadline.
    Timeout,
    /// Input or report exceeded a hard bound.
    LimitExceeded,
    /// Adapter mapping rejected a report.
    InvalidReport,
    /// No allowlisted installation adapter exists for this request.
    UnsupportedInstallation,
    /// User did not confirm this exact local plan.
    ConsentRequired,
}

/// Current availability, separate from installation and selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAvailability {
    /// Explicitly disabled; no optional process may run.
    Off,
    /// Locally available and configured.
    Available,
    /// Executable missing.
    NotFound,
    /// Detected version was invalid.
    InvalidVersion,
    /// Configuration invalid.
    ConfigurationError,
    /// Last requested execution failed.
    RuntimeFailure,
    /// Workspace security policy blocks access.
    Blocked,
}

/// Local installation state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallationStatus {
    /// Shipped as MPL-2.0 first-party functionality; cannot be uninstalled.
    BuiltIn,
    /// Local executable detected.
    Installed,
    /// No local executable detected.
    NotInstalled,
    /// Detection has not been requested.
    Unknown,
}

/// Health of the most recent refresh.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderHealth {
    /// No refresh requested yet.
    NeverRefreshed,
    /// Last refresh succeeded.
    Healthy,
    /// Last refresh failed; last valid snapshot retained.
    Stale,
    /// No valid snapshot is available after failure.
    Failed,
    /// Optional access disabled; prior snapshots must not be exposed.
    Disabled,
}

/// Monotonic refresh generation chosen by the host, not completion order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RefreshState {
    /// Most recent requested generation.
    pub generation: u64,
    /// Last accepted successful generation.
    pub last_success: Option<u64>,
    /// Current health.
    pub health: ProviderHealth,
    /// Safe class from the last attempt.
    pub error: Option<ProviderError>,
}

/// Provider catalog entry with local status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderStatus {
    /// Locally registered metadata.
    pub descriptor: ProviderDescriptor,
    /// Whether report contribution is selected. Native is always true.
    pub selected: bool,
    /// Current availability.
    pub availability: ProviderAvailability,
    /// Local installation state.
    pub installation: InstallationStatus,
    /// Last refresh state.
    pub refresh: RefreshState,
}

/// Known package source chosen by a compiled-in adapter.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PackageSource {
    /// Registry name; informational only.
    pub registry: String,
    /// Package identity; informational only.
    pub package: String,
    /// Registry URL pinned by the local adapter.
    pub url: String,
}

/// Declarative preview only. Execution adapters must reconstruct their own argv.
/// No deserialization implementation: remote metadata cannot become a plan.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InstallationPlan {
    /// Provider whose compiled-in adapter owns this plan.
    pub provider_id: String,
    /// Package source.
    pub package_source: PackageSource,
    /// Exact version requested, if independently verified.
    pub requested_version: Option<String>,
    /// Separate preview words; never a shell string or an executable authority.
    pub command_preview: Vec<String>,
    /// Expected installed executable name.
    pub expected_executable: String,
    /// Upstream license reference.
    pub license_url: String,
    /// Declared network effects.
    pub network_effects: Vec<NetworkEffect>,
    /// Declared filesystem effects.
    pub filesystem_effects: Vec<FilesystemEffect>,
    /// Whether a process restart is needed for discovery.
    pub restart_required: bool,
}

/// Network effects possible during an explicit installation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkEffect {
    /// Package registry access and dependency downloads.
    RegistryDownload,
    /// Package installation hooks may contact other destinations.
    PackageScripts,
}

/// Filesystem effects possible during an explicit installation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilesystemEffect {
    /// Package manager cache writes.
    PackageCache,
    /// User installation prefix and executable writes.
    UserInstallation,
    /// Package hooks may write with the user's privileges.
    PackageScripts,
}

/// Closed safe metric mapping. Future adapters extend this contract explicitly.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderMetric {
    /// Native source count.
    Sources,
    /// Runtime call count.
    Calls,
    /// Runtime session count.
    Sessions,
    /// Aggregate cost in USD.
    CostUsd,
    /// Aggregate token count.
    Tokens,
}

/// Fixed interpretation assumptions, never arbitrary upstream text.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderAssumption {
    /// Local source inventory at scan time.
    LocalInventory,
    /// Aggregate runtime sample, not causal effectiveness evidence.
    RuntimeAggregateNotCausal,
    /// Runtime provider's selected time window.
    ProviderWindow,
}

/// Content-free evidence reference into the Native source table.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProviderEvidence {
    /// Stable source record index within this aggregate's native report.
    pub source_index: usize,
    /// Original UTF-8 byte span.
    pub span: TextSpan,
}

/// Explicit stable fingerprint. Fixed bytes cannot accidentally contain text.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct StableFingerprint(pub [u8; 32]);

/// Safe measurement mapped by a local adapter; no raw provider strings exist.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderContribution {
    /// Metric identity within the provider namespace.
    pub metric: ProviderMetric,
    /// Finite raw measurement, not an implicitly normalized score.
    pub value: f64,
    /// Method classification.
    pub method: ScoreMethod,
    /// Required for statistical and probabilistic contributions.
    pub sample_size: Option<u64>,
    /// Fixed interpretation assumptions.
    pub assumptions: Vec<ProviderAssumption>,
    /// Local evidence locations, if applicable.
    pub evidence: Vec<ProviderEvidence>,
    /// Adapter-defined stable identity; absence prevents deduplication.
    pub fingerprint: Option<StableFingerprint>,
}

/// One provider's completed safe report. Native analysis itself stays separate.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProviderReport {
    /// Registered provider ID.
    pub provider_id: String,
    /// Refresh generation, compared by request order, not completion order.
    pub generation: u64,
    /// Safe contributed measurements.
    pub contributions: Vec<ProviderContribution>,
}

/// A deduplicated contribution retains every contributing provider namespace.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MergeProvenance {
    /// All contributing provider IDs, sorted and unique.
    pub provider_ids: Vec<String>,
    /// Unmodified contribution; conflicting values remain separate records.
    pub contribution: ProviderContribution,
}

/// Provider-neutral report envelope. Metrics are nested by provider ID;
/// no averaging, provider precedence, or writes to native analysis occur.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AggregateEnvelope {
    /// Provider contract version, independent of native report version.
    pub schema_version: u32,
    /// Selected IDs, including Native, sorted and unique.
    pub selected_providers: Vec<String>,
    /// Separate provider namespaces prevent metric collisions.
    pub reports: BTreeMap<String, ProviderReport>,
    /// Every contribution with explicit deduplication provenance.
    pub provenance: Vec<MergeProvenance>,
    /// Isolated safe errors and last valid snapshot state.
    pub refresh: BTreeMap<String, RefreshState>,
}

/// Latest requested result for a selected provider.
pub struct ProviderAttempt {
    /// Registered provider identity.
    pub provider_id: String,
    /// Host-assigned request generation.
    pub generation: u64,
    /// Mapped result or safe error.
    pub result: Result<ProviderReport, ProviderError>,
}

fn valid_report(report: &ProviderReport) -> bool {
    report.contributions.len() <= MAX_CONTRIBUTIONS && report.contributions.iter().all(|c| {
        c.value.is_finite() && c.value >= 0.0 && c.evidence.len() <= 32 && c.assumptions.len() <= 16
        && c.evidence.iter().all(|e| e.span.start <= e.span.end)
        // Probabilistic contributions are reserved until an explicit prior and
        // interval contract exists; never silently discard required uncertainty.
        && c.method != ScoreMethod::Probabilistic
        && (c.method != ScoreMethod::Statistical || c.sample_size.is_some())
        && (c.method == ScoreMethod::Deterministic || !c.assumptions.is_empty())
    })
}

/// Deterministically merges mapped provider reports. `allowed` is a compiled-in
/// catalog ID set; `retain_previous` is false on trust, root, mode or selection
/// changes. Unknown IDs are rejected before being reflected into output.
pub fn merge_reports(
    allowed: &BTreeSet<String>,
    selected: &BTreeSet<String>,
    mut attempts: Vec<ProviderAttempt>,
    previous: Option<&AggregateEnvelope>,
    retain_previous: bool,
) -> Result<AggregateEnvelope, ProviderError> {
    if allowed.len() > MAX_PROVIDERS
        || selected.len() > MAX_PROVIDERS
        || attempts.len() > MAX_PROVIDERS
    {
        return Err(ProviderError::LimitExceeded);
    }
    if !allowed.contains(NATIVE_ID)
        || !selected.is_subset(allowed)
        || attempts.iter().any(|a| !allowed.contains(&a.provider_id))
    {
        return Err(ProviderError::InvalidProvider);
    }
    let mut selected = selected.clone();
    selected.insert(NATIVE_ID.to_owned());
    attempts.sort_by(|a, b| a.provider_id.cmp(&b.provider_id));
    if attempts
        .windows(2)
        .any(|a| a[0].provider_id == a[1].provider_id)
    {
        return Err(ProviderError::InvalidReport);
    }
    let mut result = AggregateEnvelope {
        schema_version: 1,
        selected_providers: selected.iter().cloned().collect(),
        reports: BTreeMap::new(),
        provenance: Vec::new(),
        refresh: BTreeMap::new(),
    };
    for id in &selected {
        if let Some(previous) = previous.filter(|_| retain_previous) {
            if let Some(report) = previous
                .reports
                .get(id)
                .filter(|r| valid_report(r) && r.provider_id == *id)
            {
                result.reports.insert(id.clone(), report.clone());
            }
            if let Some(state) = previous.refresh.get(id) {
                result.refresh.insert(id.clone(), state.clone());
            }
        }
        result.refresh.entry(id.clone()).or_insert(RefreshState {
            generation: 0,
            last_success: None,
            health: ProviderHealth::NeverRefreshed,
            error: None,
        });
    }
    for attempt in attempts {
        if !selected.contains(&attempt.provider_id) {
            continue;
        }
        let id = attempt.provider_id;
        let state = result.refresh.get_mut(&id).expect("selected state exists");
        if attempt.generation < state.generation {
            continue;
        }
        let mapped = attempt.result.and_then(|mut r| {
            if r.provider_id != id || r.generation != attempt.generation || !valid_report(&r) {
                return Err(ProviderError::InvalidReport);
            }
            r.contributions.sort_by_cached_key(|c| {
                serde_json::to_string(c).expect("finite content-free contribution")
            });
            Ok(r)
        });
        state.generation = attempt.generation;
        match mapped {
            Ok(report) => {
                state.health = ProviderHealth::Healthy;
                state.last_success = Some(attempt.generation);
                state.error = None;
                result.reports.insert(id, report);
            }
            Err(error) => {
                if matches!(
                    error,
                    ProviderError::WorkspaceBlocked | ProviderError::ConfigurationError
                ) {
                    result.reports.remove(&id);
                }
                state.health = if result.reports.contains_key(&id) {
                    ProviderHealth::Stale
                } else {
                    ProviderHealth::Failed
                };
                state.error = Some(error);
            }
        }
    }
    let mut groups: BTreeMap<String, MergeProvenance> = BTreeMap::new();
    for (id, report) in &result.reports {
        for (index, contribution) in report.contributions.iter().enumerate() {
            let body = serde_json::to_string(contribution).expect("validated contribution");
            let key = if contribution.fingerprint.is_some() {
                format!("fingerprint:{body}")
            } else {
                format!("unique:{id}:{index:08}:{body}")
            };
            let entry = groups.entry(key).or_insert_with(|| MergeProvenance {
                provider_ids: Vec::new(),
                contribution: contribution.clone(),
            });
            if !entry.provider_ids.contains(id) {
                entry.provider_ids.push(id.clone());
            }
        }
    }
    result.provenance = groups.into_values().collect();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn contribution(value: f64) -> ProviderContribution {
        ProviderContribution {
            metric: ProviderMetric::Calls,
            value,
            method: ScoreMethod::Statistical,
            sample_size: Some(4),
            assumptions: vec![ProviderAssumption::RuntimeAggregateNotCausal],
            evidence: vec![],
            fingerprint: Some(StableFingerprint([7; 32])),
        }
    }
    fn attempt(id: &str, value: f64, generation: u64) -> ProviderAttempt {
        ProviderAttempt {
            provider_id: id.into(),
            generation,
            result: Ok(ProviderReport {
                provider_id: id.into(),
                generation,
                contributions: vec![contribution(value)],
            }),
        }
    }
    fn ids() -> BTreeSet<String> {
        [NATIVE_ID, CODEBURN_ID, "fake"].map(str::to_owned).into()
    }
    #[test]
    fn merge_order_collisions_and_explicit_fingerprint_provenance() {
        let one = merge_reports(
            &ids(),
            &ids(),
            vec![
                attempt(CODEBURN_ID, 4.0, 1),
                attempt("fake", 4.0, 1),
                attempt(NATIVE_ID, 8.0, 1),
            ],
            None,
            false,
        )
        .unwrap();
        let two = merge_reports(
            &ids(),
            &ids(),
            vec![
                attempt(NATIVE_ID, 8.0, 1),
                attempt("fake", 4.0, 1),
                attempt(CODEBURN_ID, 4.0, 1),
            ],
            None,
            false,
        )
        .unwrap();
        assert_eq!(one, two);
        assert_eq!(one.reports.len(), 3);
        assert_eq!(one.provenance.len(), 2);
        assert!(
            one.provenance
                .iter()
                .any(|p| p.provider_ids == ["codeburn", "fake"])
        );
        let mut no_fingerprint = attempt("fake", 4.0, 1);
        no_fingerprint.result.as_mut().unwrap().contributions[0].fingerprint = None;
        let report = merge_reports(
            &ids(),
            &ids(),
            vec![attempt(CODEBURN_ID, 4.0, 1), no_fingerprint],
            None,
            false,
        )
        .unwrap();
        assert_eq!(report.provenance.len(), 2);
    }
    #[test]
    fn failures_retain_only_safe_snapshots_and_never_remove_native_selection() {
        let first = merge_reports(
            &ids(),
            &ids(),
            vec![attempt(CODEBURN_ID, 4.0, 1), attempt(NATIVE_ID, 8.0, 1)],
            None,
            false,
        )
        .unwrap();
        let failed = || ProviderAttempt {
            provider_id: CODEBURN_ID.into(),
            generation: 2,
            result: Err(ProviderError::RuntimeFailure),
        };
        let next = merge_reports(&ids(), &ids(), vec![failed()], Some(&first), true).unwrap();
        assert_eq!(next.reports, first.reports);
        assert_eq!(next.refresh[CODEBURN_ID].health, ProviderHealth::Stale);
        let blocked = merge_reports(&ids(), &BTreeSet::new(), vec![], Some(&first), false).unwrap();
        assert_eq!(blocked.selected_providers, [NATIVE_ID]);
        assert!(blocked.reports.is_empty());
        let late = merge_reports(
            &ids(),
            &ids(),
            vec![attempt(CODEBURN_ID, 100.0, 0)],
            Some(&first),
            true,
        )
        .unwrap();
        assert_eq!(late.reports, first.reports);
    }
    #[test]
    fn malformed_provider_is_isolated_and_payload_fields_are_rejected() {
        let merged = merge_reports(
            &ids(),
            &ids(),
            vec![
                attempt(CODEBURN_ID, f64::NAN, 1),
                attempt(NATIVE_ID, 8.0, 1),
            ],
            None,
            false,
        )
        .unwrap();
        assert_eq!(
            merged.refresh[CODEBURN_ID].error,
            Some(ProviderError::InvalidReport)
        );
        assert!(merged.reports.contains_key(NATIVE_ID));
        assert_eq!(
            merge_reports(
                &ids(),
                &ids(),
                vec![attempt("SECRET_SENTINEL", 1.0, 1)],
                None,
                false
            ),
            Err(ProviderError::InvalidProvider)
        );
        let mut value = serde_json::to_value(contribution(4.0)).unwrap();
        for field in [
            "stdout",
            "stderr",
            "arguments",
            "transcript",
            "source",
            "credentials",
            "payload",
        ] {
            value[field] = "SECRET_SENTINEL".into();
            assert!(serde_json::from_value::<ProviderContribution>(value.clone()).is_err());
            value.as_object_mut().unwrap().remove(field);
        }
        let json = serde_json::to_string(&merged).unwrap();
        assert!(!json.contains("SECRET_SENTINEL"));
    }
}
