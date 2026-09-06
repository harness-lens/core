// SPDX-License-Identifier: MPL-2.0
// Copyright © 2026 Cristian Camargo Filho

#![doc = include_str!("../README.md")]

mod config;
mod conventions;
mod engine;
mod evaluation;
mod exact_duplicates;
mod model;
mod plugin;
pub mod statistics;
mod text_analysis;

pub use config::{
    DiscoveryConfig, EvaluationConfig, HarnessLensConfig, IntegrationConfig, PluginConfig,
};
pub use engine::{AnalysisEngine, RegistrationError};
pub use model::{
    AnalysisReport, ConfidenceEstimate, ConfiguredInputCost, EffectivenessAssessment,
    EffectivenessState, EvidenceDescriptor, Finding, FindingLocation, HarnessSource,
    HarnessSourceKind, InclusionEdge, InclusionStatus, IncompleteReason, Metric, ObservationWindow,
    ObservedCost, PluginExecution, PluginExecutionStatus, ProvenanceLink, ProvenanceRelationship,
    RuntimeAvailability, RuntimeErrorClass, RuntimeMode, RuntimeObservation,
    RuntimeObservationStatus, RuntimeReport, ScanCompleteness, ScanSummary, Score, ScoreCategory,
    ScoreError, ScoreMethod, ScoreSummary, Severity, SourceRecord, TextSpan, TokenEstimate,
};
pub use plugin::{
    IntegrationError, Plugin, PluginContext, PluginError, PluginMetadata, PluginOutput, ReportSink,
};
