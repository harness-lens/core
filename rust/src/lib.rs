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
    AnalysisReport, ConfidenceEstimate, Finding, FindingLocation, HarnessSource, HarnessSourceKind,
    IncompleteReason, Metric, PluginExecution, PluginExecutionStatus, ScanCompleteness,
    ScanSummary, Score, ScoreCategory, ScoreError, ScoreMethod, ScoreSummary, Severity,
    SourceRecord, TextSpan,
};
pub use plugin::{
    IntegrationError, Plugin, PluginContext, PluginError, PluginMetadata, PluginOutput, ReportSink,
};
