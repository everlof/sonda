use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::BTreeMap;

/// A single CLP hazard classification entry for a substance.
#[derive(Debug, Clone, Deserialize)]
pub struct HazardClass {
    /// CLP hazard class (e.g., "Carc.", "Acute Tox.", "Aquatic Acute")
    pub class: String,
    /// Category within the class (e.g., "1A", "1B", "2", "1")
    pub category: String,
    /// H-statement code (e.g., "H350", "H301", "H400")
    pub h_code: String,
    /// Exposure route if applicable (e.g., "oral", "inhalation", "dermal")
    pub route: Option<String>,
}

/// M-factors for aquatic toxicity (HP14 evaluation).
#[derive(Debug, Clone, Deserialize)]
pub struct MFactors {
    /// M-factor for acute aquatic toxicity (H400).
    pub acute: Option<Decimal>,
    /// M-factor for chronic aquatic toxicity (H410).
    pub chronic: Option<Decimal>,
}

/// A CLP harmonised substance entry, keyed by CAS number.
#[derive(Debug, Clone, Deserialize)]
pub struct ClpSubstance {
    /// Human-readable compound name.
    pub name: String,
    /// All harmonised hazard classifications.
    pub hazard_classes: Vec<HazardClass>,
    /// M-factors for aquatic toxicity.
    pub m_factors: MFactors,
    /// Specific concentration limits (SCLs) that override generic concentration limits.
    /// Key format: e.g., "Repr.1A" → Decimal threshold in % w/w.
    pub scls: BTreeMap<String, Decimal>,
}

impl ClpSubstance {
    /// Check if this substance has a specific H-code.
    pub fn has_h_code(&self, code: &str) -> bool {
        self.hazard_classes.iter().any(|hc| hc.h_code == code)
    }

    /// Check if this substance has an H-code starting with a prefix (e.g., "H350" matches "H350i").
    pub fn has_h_code_prefix(&self, prefix: &str) -> bool {
        self.hazard_classes
            .iter()
            .any(|hc| hc.h_code.starts_with(prefix))
    }

    /// Get the hazard class entry for a specific H-code.
    pub fn get_hazard_class(&self, code: &str) -> Option<&HazardClass> {
        self.hazard_classes.iter().find(|hc| hc.h_code == code)
    }

    /// Get all hazard class entries matching an H-code prefix.
    pub fn get_hazard_classes_prefix(&self, prefix: &str) -> Vec<&HazardClass> {
        self.hazard_classes
            .iter()
            .filter(|hc| hc.h_code.starts_with(prefix))
            .collect()
    }
}

/// Top-level CLP substance database.
#[derive(Debug, Clone, Deserialize)]
pub struct ClpDatabase {
    pub version: String,
    pub description: String,
    /// CAS number → substance.
    pub substances: BTreeMap<String, ClpSubstance>,
}

/// A metal speciation assumption.
#[derive(Debug, Clone, Deserialize)]
pub struct MetalSpeciation {
    /// Normalized substance name (e.g., "arsenik").
    pub substance: String,
    /// Worst-case compound name (e.g., "As2O3").
    pub compound: String,
    /// CAS number of the compound.
    pub cas: String,
    /// Conversion factor: element mass → compound mass.
    pub conversion_factor: Decimal,
    /// Explanation of the conversion factor derivation.
    pub conversion_note: Option<String>,
}

/// A PAH with direct CAS mapping (no conversion factor needed).
#[derive(Debug, Clone, Deserialize)]
pub struct PahDirect {
    /// Normalized substance name (e.g., "benso_a_pyren").
    pub substance: String,
    /// CAS number.
    pub cas: String,
}

/// Speciation assumptions table.
#[derive(Debug, Clone, Deserialize)]
pub struct SpeciationTable {
    pub version: String,
    pub description: String,
    pub metals: Vec<MetalSpeciation>,
    pub pah_direct: Vec<PahDirect>,
}
