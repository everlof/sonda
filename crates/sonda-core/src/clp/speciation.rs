use super::database;
use super::schema::ClpSubstance;
use crate::model::{AnalysisReport, AnalysisRow, AnalysisValue};
use rust_decimal::Decimal;

/// A lab-reported substance resolved to its CLP compound with concentration in % w/w.
#[derive(Debug, Clone)]
pub struct ResolvedSubstance<'a> {
    /// Original analysis row from the report.
    pub row: &'a AnalysisRow,
    /// CLP substance data.
    pub clp: &'static ClpSubstance,
    /// CAS number of the resolved compound.
    pub cas: String,
    /// Concentration in % w/w (mg/kg ÷ 10000).
    pub concentration_pct: Decimal,
    /// Whether the original value was below detection limit.
    pub below_detection: bool,
    /// Name of the CLP compound (e.g., "As2O3").
    pub compound_name: String,
}

const MGKG_TO_PCT: Decimal = Decimal::from_parts(1, 0, 0, false, 4); // 0.0001 = 1/10000

/// Resolve all substances in a report to their CLP compounds.
///
/// Returns a list of resolved substances and a list of unresolved substance names.
pub fn resolve_substances(report: &AnalysisReport) -> (Vec<ResolvedSubstance<'_>>, Vec<String>) {
    let spec_table = database::speciation_table();
    let mut resolved = Vec::new();
    let mut unresolved = Vec::new();

    for row in &report.rows {
        let name = row.normalized_name.as_str();

        // Skip PAH group sums — they have no CAS mapping
        if matches!(name, "pah_l" | "pah_m" | "pah_h" | "pah_16") {
            continue;
        }

        // Skip dry substance
        if name == "ts" {
            continue;
        }

        // Try metal speciation first
        if let Some(metal) = spec_table.metals.iter().find(|m| m.substance == name) {
            if let Some(clp) = database::lookup_by_cas(&metal.cas) {
                let (conc_pct, below_detection) = convert_to_pct(row, metal.conversion_factor);
                resolved.push(ResolvedSubstance {
                    row,
                    clp,
                    cas: metal.cas.clone(),
                    concentration_pct: conc_pct,
                    below_detection,
                    compound_name: metal.compound.clone(),
                });
                continue;
            }
        }

        // Try PAH direct mapping
        if let Some(pah) = spec_table.pah_direct.iter().find(|p| p.substance == name) {
            if let Some(clp) = database::lookup_by_cas(&pah.cas) {
                let (conc_pct, below_detection) = convert_to_pct(row, Decimal::ONE);
                resolved.push(ResolvedSubstance {
                    row,
                    clp,
                    cas: pah.cas.clone(),
                    concentration_pct: conc_pct,
                    below_detection,
                    compound_name: row.raw_name.clone(),
                });
                continue;
            }
        }

        // Unresolved — not in speciation table
        unresolved.push(name.to_string());
    }

    (resolved, unresolved)
}

/// Convert a lab value (mg/kg TS) to % w/w, applying a conversion factor.
///
/// For below-detection values, returns 0 concentration (conservative: does not contribute to sums).
/// Returns (concentration_pct, is_below_detection).
fn convert_to_pct(row: &AnalysisRow, conversion_factor: Decimal) -> (Decimal, bool) {
    match &row.value {
        AnalysisValue::Measured(val) => {
            let compound_mgkg = *val * conversion_factor;
            (compound_mgkg * MGKG_TO_PCT, false)
        }
        AnalysisValue::BelowDetection(_) => (Decimal::ZERO, true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AnalysisRow, AnalysisValue, ReportHeader, Unit};
    use rust_decimal_macros::dec;

    fn make_row(name: &str, norm: &str, val: AnalysisValue) -> AnalysisRow {
        AnalysisRow {
            raw_name: name.into(),
            normalized_name: norm.into(),
            value: val,
            unit: Unit::MgPerKgTs,
        }
    }

    fn make_report(rows: Vec<AnalysisRow>) -> AnalysisReport {
        AnalysisReport {
            header: ReportHeader::default(),
            rows,
        }
    }

    #[test]
    fn test_resolve_arsenic() {
        let report = make_report(vec![make_row(
            "Arsenik (As)",
            "arsenik",
            AnalysisValue::Measured(dec!(100)),
        )]);
        let (resolved, unresolved) = resolve_substances(&report);
        assert_eq!(resolved.len(), 1);
        assert!(unresolved.is_empty());

        let r = &resolved[0];
        assert_eq!(r.cas, "1327-53-3");
        assert_eq!(r.compound_name, "As2O3");
        // 100 mg/kg × 1.32 = 132 mg/kg → 132 / 10000 = 0.0132%
        assert_eq!(r.concentration_pct, dec!(0.0132));
        assert!(!r.below_detection);
    }

    #[test]
    fn test_resolve_bap() {
        let report = make_report(vec![make_row(
            "Benso(a)pyren",
            "benso_a_pyren",
            AnalysisValue::Measured(dec!(15)),
        )]);
        let (resolved, _) = resolve_substances(&report);
        assert_eq!(resolved.len(), 1);

        let r = &resolved[0];
        assert_eq!(r.cas, "50-32-8");
        // 15 mg/kg × 1.0 / 10000 = 0.0015%
        assert_eq!(r.concentration_pct, dec!(0.0015));
    }

    #[test]
    fn test_below_detection_is_zero() {
        let report = make_report(vec![make_row(
            "Arsenik (As)",
            "arsenik",
            AnalysisValue::BelowDetection(dec!(5)),
        )]);
        let (resolved, _) = resolve_substances(&report);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].concentration_pct, dec!(0));
        assert!(resolved[0].below_detection);
    }

    #[test]
    fn test_pah_group_sums_skipped() {
        let report = make_report(vec![
            make_row("PAH L summa", "pah_l", AnalysisValue::Measured(dec!(50))),
            make_row("PAH-16", "pah_16", AnalysisValue::Measured(dec!(200))),
        ]);
        let (resolved, _) = resolve_substances(&report);
        assert!(resolved.is_empty());
    }

    #[test]
    fn test_unknown_substance_unresolved() {
        let report = make_report(vec![make_row(
            "Bensen",
            "bensen",
            AnalysisValue::Measured(dec!(0.5)),
        )]);
        let (resolved, unresolved) = resolve_substances(&report);
        assert!(resolved.is_empty());
        assert_eq!(unresolved, vec!["bensen"]);
    }

    #[test]
    fn test_nickel_conversion() {
        let report = make_report(vec![make_row(
            "Nickel (Ni)",
            "nickel",
            AnalysisValue::Measured(dec!(1000)),
        )]);
        let (resolved, _) = resolve_substances(&report);
        assert_eq!(resolved.len(), 1);
        let r = &resolved[0];
        // 1000 mg/kg × 2.64 = 2640 mg/kg → 2640 / 10000 = 0.264%
        assert_eq!(r.concentration_pct, dec!(0.2640));
    }

    #[test]
    fn test_mixed_report() {
        let report = make_report(vec![
            make_row("Arsenik (As)", "arsenik", AnalysisValue::Measured(dec!(25))),
            make_row("Bly (Pb)", "bly", AnalysisValue::Measured(dec!(500))),
            make_row(
                "Benso(a)pyren",
                "benso_a_pyren",
                AnalysisValue::Measured(dec!(2)),
            ),
            make_row("PAH-16", "pah_16", AnalysisValue::Measured(dec!(100))),
            make_row("Bensen", "bensen", AnalysisValue::Measured(dec!(0.1))),
        ]);
        let (resolved, unresolved) = resolve_substances(&report);
        assert_eq!(resolved.len(), 3); // arsenik, bly, bap
        assert_eq!(unresolved, vec!["bensen"]);
    }
}
