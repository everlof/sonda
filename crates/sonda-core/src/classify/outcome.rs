use crate::model::AnalysisValue;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A substance's contribution to an HP criterion evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpSubstanceContribution {
    /// Normalized substance name from the report.
    pub substance: String,
    /// CLP compound name (e.g., "As2O3").
    pub compound: String,
    /// Relevant H-code for this contribution.
    pub h_code: String,
    /// Concentration in % w/w.
    pub concentration_pct: Decimal,
    /// Applicable threshold in % w/w (for individual-limit criteria).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold_pct: Option<Decimal>,
    /// Whether this individual contribution triggered the criterion.
    pub triggers: bool,
}

/// Evaluation result for a single HP criterion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpCriterionDetail {
    /// HP identifier (e.g., "HP7", "HP14").
    pub hp_id: String,
    /// HP name (e.g., "Carcinogenic", "Ecotoxic").
    pub hp_name: String,
    /// Whether this criterion was triggered.
    pub triggered: bool,
    /// Human-readable reason for the result.
    pub reason: String,
    /// Substances that contributed to the evaluation.
    pub contributions: Vec<HpSubstanceContribution>,
}

/// Full HP classification details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HpDetails {
    /// Whether the waste is classified as hazardous.
    pub is_hazardous: bool,
    /// Results for each evaluated HP criterion.
    pub criteria_results: Vec<HpCriterionDetail>,
}

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
    /// The assigned category (e.g., "KM", "MKM"), or "> MKM" if all thresholds exceeded.
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
    /// HP classification details (present only for HP-based evaluation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hp_details: Option<HpDetails>,
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
