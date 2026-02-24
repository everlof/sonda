use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisValue {
    Measured(Decimal),
    BelowDetection(Decimal),
}

impl AnalysisValue {
    /// Returns the numeric value (the measurement or the detection limit).
    pub fn numeric(&self) -> Decimal {
        match self {
            AnalysisValue::Measured(v) => *v,
            AnalysisValue::BelowDetection(v) => *v,
        }
    }

    pub fn is_below_detection(&self) -> bool {
        matches!(self, AnalysisValue::BelowDetection(_))
    }
}

impl fmt::Display for AnalysisValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisValue::Measured(v) => write!(f, "{v}"),
            AnalysisValue::BelowDetection(v) => write!(f, "< {v}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Matrix {
    Jord,
    Asfalt,
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Matrix::Jord => write!(f, "Jord"),
            Matrix::Asfalt => write!(f, "Asfalt"),
        }
    }
}

impl Matrix {
    pub fn from_str_loose(s: &str) -> Option<Matrix> {
        let lower = s.trim().to_lowercase();
        if lower.contains("jord") || lower.contains("soil") {
            Some(Matrix::Jord)
        } else if lower.contains("asfalt") || lower.contains("asphalt") {
            Some(Matrix::Asfalt)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unit {
    #[serde(rename = "mg/kg TS")]
    #[default]
    MgPerKgTs,
    #[serde(rename = "mg/kg")]
    MgPerKg,
    #[serde(rename = "%")]
    Percent,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::MgPerKgTs => write!(f, "mg/kg TS"),
            Unit::MgPerKg => write!(f, "mg/kg"),
            Unit::Percent => write!(f, "%"),
        }
    }
}

impl Unit {
    pub fn from_str_loose(s: &str) -> Unit {
        let lower = s.trim().to_lowercase();
        if lower.contains("mg/kg") && lower.contains("ts") {
            Unit::MgPerKgTs
        } else if lower.contains("mg/kg") {
            Unit::MgPerKg
        } else if lower.contains('%') {
            Unit::Percent
        } else {
            Unit::MgPerKgTs
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRow {
    pub raw_name: String,
    pub normalized_name: String,
    pub value: AnalysisValue,
    pub unit: Unit,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportHeader {
    pub lab_report_id: Option<String>,
    pub sample_id: Option<String>,
    pub matrix: Option<Matrix>,
    pub date: Option<String>,
    pub project: Option<String>,
    /// Detected laboratory (e.g., "Eurofins").
    pub lab: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub header: ReportHeader,
    pub rows: Vec<AnalysisRow>,
}
