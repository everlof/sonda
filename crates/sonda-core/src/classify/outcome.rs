use crate::model::AnalysisValue;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Classification result for a single substance against a single ruleset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstanceResult {
    /// Canonical substance name.
    pub substance: String,
    /// Name as it appeared in the report.
    pub raw_name: String,
    /// The measured or below-detection value.
    pub value: AnalysisValue,
    /// Unit string for display.
    pub unit: String,
    /// The assigned category (e.g., "KM", "MKM"), or "exceeds_all".
    pub category: String,
    /// Human-readable explanation of the classification.
    pub reason: String,
    /// The threshold that was exceeded (for the previous category), if any.
    pub exceeded_threshold: Option<Decimal>,
    /// True if classification is uncertain (detection limit >= threshold).
    pub uncertain: bool,
}

/// Classification result for one ruleset applied to a report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSetResult {
    /// Name of the ruleset that was applied.
    pub ruleset_name: String,
    /// The overall (worst) category across all substances.
    pub overall_category: String,
    /// Human-readable explanation of the overall classification.
    pub overall_reason: String,
    /// Substance(s) that determined the overall classification.
    pub determining_substances: Vec<String>,
    /// Per-substance results.
    pub substance_results: Vec<SubstanceResult>,
    /// Substances in the report that had no matching rule.
    pub unmatched_substances: Vec<String>,
    /// Rules in the ruleset that had no matching substance in the report.
    pub unmatched_rules: Vec<String>,
}

/// Classification result for a single sample (potentially multiple rulesets).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleResult {
    /// Sample identifier (Provmärkning or Provnummer).
    pub sample_id: String,
    /// Results per ruleset.
    pub ruleset_results: Vec<RuleSetResult>,
}

/// Full classification result across all samples in the PDF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub samples: Vec<SampleResult>,
}
