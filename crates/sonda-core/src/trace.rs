use crate::classify::outcome::RuleSetResult;
use crate::extraction::PageContent;
use crate::model::{AnalysisRow, AnalysisValue};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub const TRACE_SCHEMA_VERSION: &str = "1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceSeverity {
    Critical,
    Important,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceVisibility {
    Always,
    Auto,
    OnDemand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceStepType {
    ParseValue,
    NormalizeSubstance,
    ThresholdCompare,
    OverallDecision,
    HpCriterion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStep {
    pub step_type: TraceStepType,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceValueKind {
    Measured,
    BelowDetection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    pub entry_id: String,
    pub sample_id: String,
    pub raw_name: String,
    pub normalized_name: String,
    pub raw_value: String,
    pub value_kind: TraceValueKind,
    pub numeric_value: Decimal,
    pub unit: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_spans: Vec<EvidenceSpan>,
    pub steps: Vec<TraceStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceSpan {
    pub page_number: usize,
    pub line_index: usize,
    pub matched_text: String,
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceDecisionTarget {
    Substance,
    RulesetOverall,
    HpCriterion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceDecision {
    pub decision_id: String,
    pub sample_id: String,
    pub ruleset_name: String,
    pub target: TraceDecisionTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub substance: Option<String>,
    pub category: String,
    pub reason: String,
    pub severity: TraceSeverity,
    pub visibility: TraceVisibility,
    pub steps: Vec<TraceStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceWarning {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_id: Option<String>,
    pub message: String,
    pub severity: TraceSeverity,
    pub visibility: TraceVisibility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceBundle {
    pub trace_schema_version: String,
    pub entries: Vec<TraceEntry>,
    pub decisions: Vec<TraceDecision>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<TraceWarning>,
}

impl Default for TraceBundle {
    fn default() -> Self {
        Self {
            trace_schema_version: TRACE_SCHEMA_VERSION.to_string(),
            entries: Vec::new(),
            decisions: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

pub fn build_entry_trace(
    sample_id: &str,
    entry_idx: usize,
    row: &AnalysisRow,
    unit: &str,
    pages: &[PageContent],
) -> TraceEntry {
    let (value_kind, numeric_value) = match row.value {
        AnalysisValue::Measured(v) => (TraceValueKind::Measured, v),
        AnalysisValue::BelowDetection(v) => (TraceValueKind::BelowDetection, v),
    };

    TraceEntry {
        entry_id: format!("ent_{}_{}", sample_id, entry_idx),
        sample_id: sample_id.to_string(),
        raw_name: row.raw_name.clone(),
        normalized_name: row.normalized_name.clone(),
        raw_value: row.value.to_string(),
        value_kind,
        numeric_value,
        unit: unit.to_string(),
        evidence_spans: find_row_spans(pages, row),
        steps: vec![
            TraceStep {
                step_type: TraceStepType::NormalizeSubstance,
                message: format!("Normalized '{}' -> '{}'", row.raw_name, row.normalized_name),
            },
            TraceStep {
                step_type: TraceStepType::ParseValue,
                message: format!("Parsed value '{}' as {}", row.value, numeric_value),
            },
        ],
    }
}

fn find_row_spans(pages: &[PageContent], row: &AnalysisRow) -> Vec<EvidenceSpan> {
    let mut spans = Vec::new();
    let value_text = row.value.to_string();

    for page in pages {
        for span in &page.line_spans {
            let line_lower = span.text.to_lowercase();
            let name_match = line_lower.contains(&row.raw_name.to_lowercase());
            let value_match = span.text.contains(&value_text);

            if name_match || value_match {
                spans.push(EvidenceSpan {
                    page_number: span.page_number,
                    line_index: span.line_index,
                    matched_text: span.text.clone(),
                    x_min: span.bbox.x_min,
                    y_min: span.bbox.y_min,
                    x_max: span.bbox.x_max,
                    y_max: span.bbox.y_max,
                });
            }
        }
    }

    spans
}

pub fn build_ruleset_decisions(
    sample_id: &str,
    ruleset_idx: usize,
    rs: &RuleSetResult,
) -> Vec<TraceDecision> {
    let mut decisions = Vec::new();

    decisions.push(TraceDecision {
        decision_id: format!("dec_{}_{}_overall", sample_id, ruleset_idx),
        sample_id: sample_id.to_string(),
        ruleset_name: rs.ruleset_name.clone(),
        target: TraceDecisionTarget::RulesetOverall,
        substance: None,
        category: rs.overall_category.clone(),
        reason: rs.overall_reason.clone(),
        severity: TraceSeverity::Important,
        visibility: TraceVisibility::Always,
        steps: vec![TraceStep {
            step_type: TraceStepType::OverallDecision,
            message: format!(
                "Overall category '{}' determined by: {}",
                rs.overall_category,
                if rs.determining_substances.is_empty() {
                    "none".to_string()
                } else {
                    rs.determining_substances.join(", ")
                }
            ),
        }],
    });

    for (sub_idx, sr) in rs.substance_results.iter().enumerate() {
        decisions.push(TraceDecision {
            decision_id: format!("dec_{}_{}_sub_{}", sample_id, ruleset_idx, sub_idx),
            sample_id: sample_id.to_string(),
            ruleset_name: rs.ruleset_name.clone(),
            target: TraceDecisionTarget::Substance,
            substance: Some(sr.substance.clone()),
            category: sr.category.clone(),
            reason: sr.reason.clone(),
            severity: if sr.uncertain {
                TraceSeverity::Important
            } else {
                TraceSeverity::Info
            },
            visibility: TraceVisibility::Auto,
            steps: vec![TraceStep {
                step_type: if rs.hp_details.is_some() {
                    TraceStepType::HpCriterion
                } else {
                    TraceStepType::ThresholdCompare
                },
                message: sr.reason.clone(),
            }],
        });
    }

    decisions
}
