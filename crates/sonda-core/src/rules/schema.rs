use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A ruleset defining classification thresholds for chemical substances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSetDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub version: String,
    /// If set, this ruleset only applies to reports with this matrix.
    #[serde(default)]
    pub matrix: Option<String>,
    /// Ordered list of categories, from cleanest to most contaminated.
    pub categories: Vec<String>,
    #[serde(default)]
    pub category_descriptions: BTreeMap<String, String>,
    pub rules: Vec<SubstanceRuleDef>,
}

/// A single substance rule within a ruleset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstanceRuleDef {
    pub substance: String,
    /// Map of category name -> threshold value (as string for exact decimal).
    pub thresholds: BTreeMap<String, Decimal>,
    #[serde(default)]
    pub matrix: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}
