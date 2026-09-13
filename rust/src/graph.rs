// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

//! Versioned provider-neutral relationship-graph contracts.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::trace::{
    EvidenceCompleteness, EvidenceLocation, TraceValidationError, validate_identifier,
    validate_label, validate_window,
};
use crate::{ObservationWindow, RuntimeObservationStatus, ScoreMethod};

/// Current relationship-graph schema version.
pub const RELATIONSHIP_GRAPH_SCHEMA_VERSION: u32 = 1;

/// Evidence semantics represented by one graph envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphKind {
    /// Proven static relationships.
    StaticRelationship,
    /// Bounded possible/static paths, never observed behavior.
    PossiblePath,
    /// Ordered runtime transitions backed by observed samples.
    ObservedFlow,
}

/// Stable semantic category for graph nodes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphNodeKind {
    /// Harness file or configuration asset.
    Harness,
    /// Skill asset.
    Skill,
    /// Rule or finding identity.
    Rule,
    /// Observed action or tool identity.
    Action,
    /// Stable action category.
    Category,
    /// Terminal status.
    Status,
    /// Extensible provider-neutral node.
    Other,
}

/// Relationship semantics for one directed edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphRelationship {
    /// Proven static relationship.
    Static,
    /// Possible/static transition.
    Possible,
    /// Ordered observed transition.
    ObservedTransition,
}

/// Fixed processing limits recorded in every graph response.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GraphLimits {
    /// Maximum serialized nodes.
    pub max_nodes: usize,
    /// Maximum serialized edges.
    pub max_edges: usize,
    /// Maximum directed hops from a selected root.
    pub max_hops: usize,
}

/// Filters applied before graph bounds.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphFilters {
    /// Canonical logical root identity, when selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    /// Inclusive observation window filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<ObservationWindow>,
    /// Stable selected categories in lexical order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    /// Selected terminal statuses in enum order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<RuntimeObservationStatus>,
    /// Minimum pre-bound transition share.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_share: Option<f64>,
    /// Exactly one metric unit used for every visible edge.
    pub metric_unit: String,
}

/// Bounded provenance references for one node or edge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphProvenance {
    /// Stable adapter or analysis source.
    pub source: String,
    /// Evidence method.
    pub method: ScoreMethod,
    /// Canonically ordered safe observation or finding identities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_ids: Vec<String>,
    /// Total evidence references before this list's bound.
    pub total_evidence: usize,
    /// Safe navigation location, when attributable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<EvidenceLocation>,
}

impl GraphProvenance {
    fn canonicalize(&mut self) {
        self.evidence_ids.sort();
        self.evidence_ids.dedup();
    }

    fn validate(&self) -> Result<(), GraphValidationError> {
        validate_identifier("graph provenance source", &self.source)?;
        if self.total_evidence < self.evidence_ids.len() {
            return Err(GraphValidationError::InvalidProvenance);
        }
        let mut seen = BTreeSet::new();
        let mut previous: Option<&str> = None;
        for id in &self.evidence_ids {
            validate_identifier("graph evidence id", id)?;
            if !seen.insert(id) || previous.is_some_and(|value| value >= id.as_str()) {
                return Err(GraphValidationError::NonCanonicalOrder("evidence ids"));
            }
            previous = Some(id);
        }
        if let Some(location) = &self.location {
            location.validate()?;
        }
        Ok(())
    }
}

/// One graph node. Layered copies preserve the same `logical_id` across cycles.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique serialized node identity.
    pub id: String,
    /// Canonical logical entity identity shared across layers.
    pub logical_id: String,
    /// Content-safe display label.
    pub label: String,
    /// Semantic node kind.
    pub kind: GraphNodeKind,
    /// Sequence layer used by observed-flow renderers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer: Option<usize>,
    /// Content-safe evidence references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provenance: Vec<GraphProvenance>,
}

/// Declared width metric for one observed-flow edge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeightedEdgeMetric {
    /// Non-negative measured value used for width.
    pub value: f64,
    /// Explicit metric unit shared by all graph edges.
    pub unit: String,
    /// Positive total after active filters and before response truncation.
    pub denominator: f64,
    /// `value / denominator`, normalized to 0.0 through 1.0.
    pub share: f64,
    /// Number of observed transitions contributing to this edge.
    pub sample_size: usize,
    /// Inclusive observation window.
    pub window: ObservationWindow,
}

impl WeightedEdgeMetric {
    fn validate(&self) -> Result<(), GraphValidationError> {
        validate_identifier("weighted edge unit", &self.unit)?;
        validate_window(&self.window)?;
        if !self.value.is_finite()
            || self.value < 0.0
            || !self.denominator.is_finite()
            || self.denominator <= 0.0
            || !self.share.is_finite()
            || !(0.0..=1.0).contains(&self.share)
            || self.sample_size == 0
        {
            return Err(GraphValidationError::InvalidMetric);
        }
        let expected = self.value / self.denominator;
        if (expected - self.share).abs() > 1e-9 {
            return Err(GraphValidationError::InvalidMetric);
        }
        Ok(())
    }
}

/// One directed graph edge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Stable edge identity.
    pub id: String,
    /// Serialized source node identity.
    pub source: String,
    /// Serialized target node identity.
    pub target: String,
    /// Relationship semantics.
    pub relationship: GraphRelationship,
    /// Method used to establish this edge.
    pub method: ScoreMethod,
    /// Width metric, required only for observed flow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric: Option<WeightedEdgeMetric>,
    /// Content-safe evidence references.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provenance: Vec<GraphProvenance>,
}

/// Versioned, bounded provider-neutral relationship graph.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RelationshipGraph {
    /// Graph schema version.
    pub schema_version: u32,
    /// Evidence semantics represented by this graph.
    pub kind: GraphKind,
    /// Graph-wide method classification.
    pub method: ScoreMethod,
    /// Visible completeness and truncation state.
    pub completeness: EvidenceCompleteness,
    /// Bounds applied by the producer.
    pub limits: GraphLimits,
    /// Active filters and selected metric unit.
    pub filters: GraphFilters,
    /// Canonically ordered nodes.
    #[serde(default)]
    pub nodes: Vec<GraphNode>,
    /// Canonically ordered directed edges.
    #[serde(default)]
    pub edges: Vec<GraphEdge>,
}

impl RelationshipGraph {
    /// Sorts all set-like fields into deterministic serialized order.
    #[must_use]
    pub fn canonicalize(mut self) -> Self {
        self.completeness.canonicalize();
        self.filters.categories.sort();
        self.filters.categories.dedup();
        self.filters.statuses.sort();
        self.filters.statuses.dedup();
        for node in &mut self.nodes {
            for provenance in &mut node.provenance {
                provenance.canonicalize();
            }
            node.provenance.sort_by(|left, right| {
                (&left.source, &left.evidence_ids).cmp(&(&right.source, &right.evidence_ids))
            });
        }
        for edge in &mut self.edges {
            for provenance in &mut edge.provenance {
                provenance.canonicalize();
            }
            edge.provenance.sort_by(|left, right| {
                (&left.source, &left.evidence_ids).cmp(&(&right.source, &right.evidence_ids))
            });
        }
        self.nodes.sort_by(|left, right| left.id.cmp(&right.id));
        self.edges.sort_by(|left, right| left.id.cmp(&right.id));
        self
    }

    /// Validates version, bounds, ordering, endpoints, evidence, and graph-kind separation.
    pub fn validate(&self) -> Result<(), GraphValidationError> {
        if self.schema_version != RELATIONSHIP_GRAPH_SCHEMA_VERSION {
            return Err(GraphValidationError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        if self.limits.max_nodes == 0 || self.limits.max_edges == 0 || self.limits.max_hops == 0 {
            return Err(GraphValidationError::InvalidLimits);
        }
        if self.nodes.len() > self.limits.max_nodes {
            return Err(GraphValidationError::BoundExceeded {
                kind: "nodes",
                limit: self.limits.max_nodes,
                actual: self.nodes.len(),
            });
        }
        if self.edges.len() > self.limits.max_edges {
            return Err(GraphValidationError::BoundExceeded {
                kind: "edges",
                limit: self.limits.max_edges,
                actual: self.edges.len(),
            });
        }
        self.completeness.validate()?;
        self.validate_filters()?;

        let mut ids = BTreeSet::new();
        let mut logical_layers = BTreeSet::new();
        let mut previous: Option<&str> = None;
        for node in &self.nodes {
            validate_identifier("graph node id", &node.id)?;
            validate_identifier("graph logical node id", &node.logical_id)?;
            validate_label("graph node label", &node.label)?;
            if self.kind == GraphKind::ObservedFlow && node.layer.is_none() {
                return Err(GraphValidationError::MissingLayer);
            }
            if !ids.insert(node.id.as_str()) {
                return Err(GraphValidationError::DuplicateNode);
            }
            if !logical_layers.insert((node.logical_id.as_str(), node.layer)) {
                return Err(GraphValidationError::DuplicateLogicalLayer);
            }
            if previous.is_some_and(|value| value >= node.id.as_str()) {
                return Err(GraphValidationError::NonCanonicalOrder("nodes"));
            }
            previous = Some(&node.id);
            for provenance in &node.provenance {
                provenance.validate()?;
            }
        }

        let mut edge_ids = BTreeSet::new();
        let mut previous: Option<&str> = None;
        for edge in &self.edges {
            validate_identifier("graph edge id", &edge.id)?;
            if !edge_ids.insert(edge.id.as_str()) {
                return Err(GraphValidationError::DuplicateEdge);
            }
            if previous.is_some_and(|value| value >= edge.id.as_str()) {
                return Err(GraphValidationError::NonCanonicalOrder("edges"));
            }
            previous = Some(&edge.id);
            if !ids.contains(edge.source.as_str()) || !ids.contains(edge.target.as_str()) {
                return Err(GraphValidationError::MissingEndpoint);
            }
            match self.kind {
                GraphKind::StaticRelationship
                    if edge.relationship != GraphRelationship::Static || edge.metric.is_some() =>
                {
                    return Err(GraphValidationError::KindMismatch);
                }
                GraphKind::PossiblePath
                    if edge.relationship != GraphRelationship::Possible
                        || edge.metric.is_some() =>
                {
                    return Err(GraphValidationError::KindMismatch);
                }
                GraphKind::ObservedFlow
                    if edge.relationship != GraphRelationship::ObservedTransition
                        || edge.method != ScoreMethod::Statistical
                        || edge.metric.is_none() =>
                {
                    return Err(GraphValidationError::KindMismatch);
                }
                _ => {}
            }
            if let Some(metric) = &edge.metric {
                metric.validate()?;
                if metric.unit != self.filters.metric_unit {
                    return Err(GraphValidationError::KindMismatch);
                }
            }
            if self.kind == GraphKind::ObservedFlow
                && (edge.provenance.is_empty()
                    || edge
                        .provenance
                        .iter()
                        .all(|provenance| provenance.total_evidence == 0))
            {
                return Err(GraphValidationError::InvalidProvenance);
            }
            for provenance in &edge.provenance {
                provenance.validate()?;
            }
        }
        Ok(())
    }

    fn validate_filters(&self) -> Result<(), GraphValidationError> {
        validate_identifier("graph metric unit", &self.filters.metric_unit)?;
        if let Some(root) = &self.filters.root {
            validate_identifier("graph root", root)?;
        }
        if let Some(window) = &self.filters.window {
            validate_window(window)?;
        }
        if self
            .filters
            .minimum_share
            .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            return Err(GraphValidationError::InvalidMetric);
        }
        let mut previous: Option<&str> = None;
        for category in &self.filters.categories {
            validate_identifier("graph category filter", category)?;
            if previous.is_some_and(|value| value >= category.as_str()) {
                return Err(GraphValidationError::NonCanonicalOrder("category filters"));
            }
            previous = Some(category);
        }
        let mut seen = BTreeSet::new();
        for status in &self.filters.statuses {
            if !seen.insert(status) {
                return Err(GraphValidationError::NonCanonicalOrder("status filters"));
            }
        }
        if !self
            .filters
            .statuses
            .windows(2)
            .all(|pair| pair[0] < pair[1])
        {
            return Err(GraphValidationError::NonCanonicalOrder("status filters"));
        }
        Ok(())
    }
}

/// Invalid relationship-graph contract.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphValidationError {
    /// Schema version is unsupported.
    UnsupportedSchemaVersion(u32),
    /// A declared graph limit is zero.
    InvalidLimits,
    /// A bounded collection exceeded its declared limit.
    BoundExceeded {
        /// Collection name.
        kind: &'static str,
        /// Declared limit.
        limit: usize,
        /// Actual collection length.
        actual: usize,
    },
    /// Node identity appears more than once.
    DuplicateNode,
    /// Logical identity appears more than once in one layer.
    DuplicateLogicalLayer,
    /// Edge identity appears more than once.
    DuplicateEdge,
    /// Edge endpoint does not name a visible node.
    MissingEndpoint,
    /// Observed-flow node lacks an explicit sequence layer.
    MissingLayer,
    /// Graph kind, relationship, method, or metric disagree.
    KindMismatch,
    /// Weighted metric is invalid.
    InvalidMetric,
    /// Provenance count, identity, or location is invalid.
    InvalidProvenance,
    /// Serialized collection is not in canonical order.
    NonCanonicalOrder(&'static str),
    /// Shared trace/evidence validation failed.
    Trace(TraceValidationError),
}

impl fmt::Display for GraphValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(version) => {
                write!(
                    formatter,
                    "unsupported relationship-graph schema version: {version}"
                )
            }
            Self::InvalidLimits => formatter.write_str("graph limits must be nonzero"),
            Self::BoundExceeded {
                kind,
                limit,
                actual,
            } => write!(formatter, "{kind} count {actual} exceeds limit {limit}"),
            Self::DuplicateNode => formatter.write_str("duplicate graph node identity"),
            Self::DuplicateLogicalLayer => {
                formatter.write_str("duplicate logical node identity in sequence layer")
            }
            Self::DuplicateEdge => formatter.write_str("duplicate graph edge identity"),
            Self::MissingEndpoint => formatter.write_str("graph edge endpoint is missing"),
            Self::MissingLayer => formatter.write_str("observed-flow node has no sequence layer"),
            Self::KindMismatch => formatter.write_str("graph kind and edge semantics disagree"),
            Self::InvalidMetric => formatter.write_str("invalid weighted edge metric"),
            Self::InvalidProvenance => formatter.write_str("invalid graph provenance"),
            Self::NonCanonicalOrder(kind) => {
                write!(formatter, "{kind} are not canonically ordered")
            }
            Self::Trace(error) => write!(formatter, "invalid shared evidence: {error}"),
        }
    }
}

impl Error for GraphValidationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Trace(error) => Some(error),
            _ => None,
        }
    }
}

impl From<TraceValidationError> for GraphValidationError {
    fn from(value: TraceValidationError) -> Self {
        Self::Trace(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph() -> RelationshipGraph {
        let window = ObservationWindow {
            start: "2026-09-13T00:00:00Z".to_owned(),
            end: "2026-09-13T01:00:00Z".to_owned(),
        };
        RelationshipGraph {
            schema_version: RELATIONSHIP_GRAPH_SCHEMA_VERSION,
            kind: GraphKind::ObservedFlow,
            method: ScoreMethod::Statistical,
            completeness: EvidenceCompleteness::default(),
            limits: GraphLimits {
                max_nodes: 10,
                max_edges: 10,
                max_hops: 10,
            },
            filters: GraphFilters {
                metric_unit: "transitions".to_owned(),
                ..GraphFilters::default()
            },
            nodes: vec![
                GraphNode {
                    id: "layer:2:tool.read".to_owned(),
                    logical_id: "tool.read".to_owned(),
                    label: "Read".to_owned(),
                    kind: GraphNodeKind::Action,
                    layer: Some(2),
                    provenance: vec![GraphProvenance {
                        source: "harness-lens-sdk".to_owned(),
                        method: ScoreMethod::Statistical,
                        evidence_ids: vec!["obs-3".to_owned()],
                        total_evidence: 1,
                        location: None,
                    }],
                },
                GraphNode {
                    id: "layer:1:tool.write".to_owned(),
                    logical_id: "tool.write".to_owned(),
                    label: "Write".to_owned(),
                    kind: GraphNodeKind::Action,
                    layer: Some(1),
                    provenance: vec![GraphProvenance {
                        source: "harness-lens-sdk".to_owned(),
                        method: ScoreMethod::Statistical,
                        evidence_ids: vec!["obs-2".to_owned()],
                        total_evidence: 1,
                        location: None,
                    }],
                },
                GraphNode {
                    id: "layer:3:tool.write".to_owned(),
                    logical_id: "tool.write".to_owned(),
                    label: "Write".to_owned(),
                    kind: GraphNodeKind::Action,
                    layer: Some(3),
                    provenance: vec![GraphProvenance {
                        source: "harness-lens-sdk".to_owned(),
                        method: ScoreMethod::Statistical,
                        evidence_ids: vec!["obs-3".to_owned()],
                        total_evidence: 1,
                        location: None,
                    }],
                },
            ],
            edges: vec![
                GraphEdge {
                    id: "edge:2".to_owned(),
                    source: "layer:2:tool.read".to_owned(),
                    target: "layer:3:tool.write".to_owned(),
                    relationship: GraphRelationship::ObservedTransition,
                    method: ScoreMethod::Statistical,
                    metric: Some(WeightedEdgeMetric {
                        value: 1.0,
                        unit: "transitions".to_owned(),
                        denominator: 2.0,
                        share: 0.5,
                        sample_size: 1,
                        window: window.clone(),
                    }),
                    provenance: vec![GraphProvenance {
                        source: "harness-lens-sdk".to_owned(),
                        method: ScoreMethod::Statistical,
                        evidence_ids: vec!["obs-2".to_owned()],
                        total_evidence: 1,
                        location: None,
                    }],
                },
                GraphEdge {
                    id: "edge:1".to_owned(),
                    source: "layer:1:tool.write".to_owned(),
                    target: "layer:2:tool.read".to_owned(),
                    relationship: GraphRelationship::ObservedTransition,
                    method: ScoreMethod::Statistical,
                    metric: Some(WeightedEdgeMetric {
                        value: 1.0,
                        unit: "transitions".to_owned(),
                        denominator: 2.0,
                        share: 0.5,
                        sample_size: 1,
                        window,
                    }),
                    provenance: vec![GraphProvenance {
                        source: "harness-lens-sdk".to_owned(),
                        method: ScoreMethod::Statistical,
                        evidence_ids: vec!["obs-2".to_owned()],
                        total_evidence: 1,
                        location: None,
                    }],
                },
            ],
        }
        .canonicalize()
    }

    #[test]
    fn observed_flow_preserves_repeated_logical_nodes_across_layers() {
        let graph = graph();
        graph.validate().unwrap();
        let repeated = graph
            .nodes
            .iter()
            .filter(|node| node.logical_id == "tool.write")
            .collect::<Vec<_>>();
        assert_eq!(repeated.len(), 2);
        assert_ne!(repeated[0].layer, repeated[1].layer);

        let first = serde_json::to_string(&graph).unwrap();
        let second = serde_json::to_string(
            &serde_json::from_str::<RelationshipGraph>(&first)
                .unwrap()
                .canonicalize(),
        )
        .unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn observed_flow_rejects_missing_endpoints_and_implicit_widths() {
        let mut missing = graph();
        missing.edges[0].target = "layer:9:missing".to_owned();
        assert_eq!(
            missing.validate(),
            Err(GraphValidationError::MissingEndpoint)
        );

        let mut metric = graph();
        metric.edges[0].metric = None;
        assert_eq!(metric.validate(), Err(GraphValidationError::KindMismatch));
    }

    #[test]
    fn empty_partial_graph_remains_explicit() {
        let graph = RelationshipGraph {
            schema_version: RELATIONSHIP_GRAPH_SCHEMA_VERSION,
            kind: GraphKind::ObservedFlow,
            method: ScoreMethod::Statistical,
            completeness: EvidenceCompleteness {
                complete: false,
                reasons: vec![crate::CompletenessReason {
                    code: "unavailable".to_owned(),
                    count: None,
                }],
            },
            limits: GraphLimits {
                max_nodes: 1,
                max_edges: 1,
                max_hops: 1,
            },
            filters: GraphFilters {
                metric_unit: "transitions".to_owned(),
                ..GraphFilters::default()
            },
            nodes: Vec::new(),
            edges: Vec::new(),
        };
        graph.validate().unwrap();
    }
}
