use crate::classify::outcome::{
    HpCriterionDetail, HpDetails, HpSubstanceContribution, RuleSetResult, SubstanceResult,
};
use crate::clp::schema::ClpSubstance;
use crate::clp::speciation::{resolve_substances, ResolvedSubstance};
use crate::model::AnalysisReport;
use rust_decimal::Decimal;

const ONE: Decimal = Decimal::ONE;

/// Classify a report using HP criteria (EU Regulation 1357/2014 and 2017/997).
///
/// Returns a `RuleSetResult` with FA/Icke FA classification and HP details.
pub fn classify_hp(report: &AnalysisReport) -> RuleSetResult {
    let (resolved, unresolved) = resolve_substances(report);

    let criteria_results = vec![
        evaluate_hp7(&resolved),
        evaluate_hp11(&resolved),
        evaluate_hp10(&resolved),
        evaluate_hp5(&resolved),
        evaluate_hp6(&resolved),
        evaluate_hp4(&resolved),
        evaluate_hp8(&resolved),
        evaluate_hp13(&resolved),
        evaluate_hp14(&resolved),
    ];

    let is_hazardous = criteria_results.iter().any(|c| c.triggered);

    let triggered_ids: Vec<&str> = criteria_results
        .iter()
        .filter(|c| c.triggered)
        .map(|c| c.hp_id.as_str())
        .collect();

    let overall_category = if is_hazardous {
        "FA".to_string()
    } else {
        "Icke FA".to_string()
    };

    let overall_reason = if is_hazardous {
        format!("Farligt avfall: triggered by {}", triggered_ids.join(", "))
    } else {
        "Icke farligt avfall: no HP criteria triggered".to_string()
    };

    let determining_substances: Vec<String> = criteria_results
        .iter()
        .filter(|c| c.triggered)
        .flat_map(|c| {
            c.contributions
                .iter()
                .filter(|s| s.triggers)
                .map(|s| s.substance.clone())
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    // Build per-substance results
    let substance_results: Vec<SubstanceResult> = resolved
        .iter()
        .map(|r| {
            let contributes_to_fa = criteria_results.iter().any(|cr| {
                cr.triggered
                    && cr
                        .contributions
                        .iter()
                        .any(|c| c.substance == r.row.normalized_name && c.triggers)
            });

            SubstanceResult {
                substance: r.row.normalized_name.clone(),
                raw_name: r.row.raw_name.clone(),
                value: r.row.value.clone(),
                unit: "mg/kg TS".to_string(),
                category: if contributes_to_fa {
                    "FA".to_string()
                } else {
                    "Icke FA".to_string()
                },
                reason: format!(
                    "{} -> {} ({}): {:.4}% w/w",
                    r.row.raw_name, r.compound_name, r.cas, r.concentration_pct,
                ),
                exceeded_threshold: None,
                uncertain: false,
            }
        })
        .collect();

    let hp_details = HpDetails {
        is_hazardous,
        criteria_results,
    };

    RuleSetResult {
        ruleset_name: "Farligt avfall (HP-bedömning)".to_string(),
        overall_category,
        overall_reason,
        lowest_category: None,
        determining_substances,
        substance_results,
        unmatched_substances: unresolved,
        unmatched_rules: vec![],
        hp_details: Some(hp_details),
    }
}

// ---------------------------------------------------------------------------
// HP7: Carcinogenic
// Individual limits: H350 (1A/1B) ≥ 0.1%, H351 (2) ≥ 1.0%
// ---------------------------------------------------------------------------

fn evaluate_hp7(resolved: &[ResolvedSubstance<'_>]) -> HpCriterionDetail {
    let mut contributions = Vec::new();
    let mut triggered = false;

    for r in resolved {
        if r.below_detection {
            continue;
        }

        // H350 or H350i → Carc. 1A/1B → threshold 0.1%
        if let Some(hc) = r
            .clp
            .hazard_classes
            .iter()
            .find(|hc| hc.h_code.starts_with("H350"))
        {
            let threshold = dec_const("0.1");
            let triggers = r.concentration_pct >= threshold;
            if triggers {
                triggered = true;
            }
            contributions.push(HpSubstanceContribution {
                substance: r.row.normalized_name.clone(),
                compound: r.compound_name.clone(),
                h_code: hc.h_code.clone(),
                concentration_pct: r.concentration_pct,
                threshold_pct: Some(threshold),
                triggers,
            });
        }

        // H351 → Carc. 2 → threshold 1.0%
        if r.clp.has_h_code("H351") {
            let threshold = dec_const("1.0");
            let triggers = r.concentration_pct >= threshold;
            if triggers {
                triggered = true;
            }
            contributions.push(HpSubstanceContribution {
                substance: r.row.normalized_name.clone(),
                compound: r.compound_name.clone(),
                h_code: "H351".to_string(),
                concentration_pct: r.concentration_pct,
                threshold_pct: Some(threshold),
                triggers,
            });
        }
    }

    let reason = if triggered {
        "One or more substances exceed individual carcinogenic concentration limits".to_string()
    } else {
        "No substances exceed carcinogenic concentration limits".to_string()
    };

    HpCriterionDetail {
        hp_id: "HP7".to_string(),
        hp_name: "Carcinogenic".to_string(),
        triggered,
        reason,
        contributions,
    }
}

// ---------------------------------------------------------------------------
// HP11: Mutagenic
// Individual limits: H340 (1A/1B) ≥ 0.1%, H341 (2) ≥ 1.0%
// ---------------------------------------------------------------------------

fn evaluate_hp11(resolved: &[ResolvedSubstance<'_>]) -> HpCriterionDetail {
    let mut contributions = Vec::new();
    let mut triggered = false;

    for r in resolved {
        if r.below_detection {
            continue;
        }

        if r.clp.has_h_code("H340") {
            let threshold = dec_const("0.1");
            let triggers = r.concentration_pct >= threshold;
            if triggers {
                triggered = true;
            }
            contributions.push(HpSubstanceContribution {
                substance: r.row.normalized_name.clone(),
                compound: r.compound_name.clone(),
                h_code: "H340".to_string(),
                concentration_pct: r.concentration_pct,
                threshold_pct: Some(threshold),
                triggers,
            });
        }

        if r.clp.has_h_code("H341") {
            let threshold = dec_const("1.0");
            let triggers = r.concentration_pct >= threshold;
            if triggers {
                triggered = true;
            }
            contributions.push(HpSubstanceContribution {
                substance: r.row.normalized_name.clone(),
                compound: r.compound_name.clone(),
                h_code: "H341".to_string(),
                concentration_pct: r.concentration_pct,
                threshold_pct: Some(threshold),
                triggers,
            });
        }
    }

    let reason = if triggered {
        "One or more substances exceed individual mutagenic concentration limits".to_string()
    } else {
        "No substances exceed mutagenic concentration limits".to_string()
    };

    HpCriterionDetail {
        hp_id: "HP11".to_string(),
        hp_name: "Mutagenic".to_string(),
        triggered,
        reason,
        contributions,
    }
}

// ---------------------------------------------------------------------------
// HP10: Toxic for reproduction
// Individual limits: H360 (1A/1B) ≥ 0.3% (or SCL), H361 (2) ≥ 0.3%
// ---------------------------------------------------------------------------

fn evaluate_hp10(resolved: &[ResolvedSubstance<'_>]) -> HpCriterionDetail {
    let mut contributions = Vec::new();
    let mut triggered = false;

    for r in resolved {
        if r.below_detection {
            continue;
        }

        // H360 variants (H360FD, H360D, H360F, etc.)
        if let Some(hc) = r
            .clp
            .hazard_classes
            .iter()
            .find(|hc| hc.h_code.starts_with("H360"))
        {
            // Check for SCL override
            let default_threshold = dec_const("0.3");
            let threshold = get_scl_for_repr(r.clp, "1A")
                .or_else(|| get_scl_for_repr(r.clp, "1B"))
                .unwrap_or(default_threshold);

            let triggers = r.concentration_pct >= threshold;
            if triggers {
                triggered = true;
            }
            contributions.push(HpSubstanceContribution {
                substance: r.row.normalized_name.clone(),
                compound: r.compound_name.clone(),
                h_code: hc.h_code.clone(),
                concentration_pct: r.concentration_pct,
                threshold_pct: Some(threshold),
                triggers,
            });
        }

        // H361 variants
        if let Some(hc) = r
            .clp
            .hazard_classes
            .iter()
            .find(|hc| hc.h_code.starts_with("H361"))
        {
            let threshold = dec_const("0.3");
            let triggers = r.concentration_pct >= threshold;
            if triggers {
                triggered = true;
            }
            contributions.push(HpSubstanceContribution {
                substance: r.row.normalized_name.clone(),
                compound: r.compound_name.clone(),
                h_code: hc.h_code.clone(),
                concentration_pct: r.concentration_pct,
                threshold_pct: Some(threshold),
                triggers,
            });
        }
    }

    let reason = if triggered {
        "One or more substances exceed reproductive toxicity concentration limits".to_string()
    } else {
        "No substances exceed reproductive toxicity concentration limits".to_string()
    };

    HpCriterionDetail {
        hp_id: "HP10".to_string(),
        hp_name: "Toxic for reproduction".to_string(),
        triggered,
        reason,
        contributions,
    }
}

// ---------------------------------------------------------------------------
// HP5: STOT SE/RE + Aspiration
// Individual: H370 ≥ 1.0%, H371 ≥ 10.0%
// Summation: H372 sum ≥ 1.0%, H373 sum ≥ 10.0%
// ---------------------------------------------------------------------------

fn evaluate_hp5(resolved: &[ResolvedSubstance<'_>]) -> HpCriterionDetail {
    let mut contributions = Vec::new();
    let mut triggered = false;

    let threshold_h370 = dec_const("1.0");
    let threshold_h371 = dec_const("10.0");
    let threshold_h372 = dec_const("1.0");
    let threshold_h373 = dec_const("10.0");

    // Individual: H370
    for r in resolved {
        if r.below_detection {
            continue;
        }
        if r.clp.has_h_code("H370") {
            let triggers = r.concentration_pct >= threshold_h370;
            if triggers {
                triggered = true;
            }
            contributions.push(HpSubstanceContribution {
                substance: r.row.normalized_name.clone(),
                compound: r.compound_name.clone(),
                h_code: "H370".to_string(),
                concentration_pct: r.concentration_pct,
                threshold_pct: Some(threshold_h370),
                triggers,
            });
        }
    }

    // Individual: H371
    for r in resolved {
        if r.below_detection {
            continue;
        }
        if r.clp.has_h_code("H371") {
            let triggers = r.concentration_pct >= threshold_h371;
            if triggers {
                triggered = true;
            }
            contributions.push(HpSubstanceContribution {
                substance: r.row.normalized_name.clone(),
                compound: r.compound_name.clone(),
                h_code: "H371".to_string(),
                concentration_pct: r.concentration_pct,
                threshold_pct: Some(threshold_h371),
                triggers,
            });
        }
    }

    // Summation: H372
    let sum_h372: Decimal = resolved
        .iter()
        .filter(|r| !r.below_detection && r.clp.has_h_code("H372"))
        .map(|r| r.concentration_pct)
        .sum();

    if sum_h372 >= threshold_h372 {
        triggered = true;
    }

    for r in resolved {
        if r.below_detection {
            continue;
        }
        if r.clp.has_h_code("H372") {
            contributions.push(HpSubstanceContribution {
                substance: r.row.normalized_name.clone(),
                compound: r.compound_name.clone(),
                h_code: "H372".to_string(),
                concentration_pct: r.concentration_pct,
                threshold_pct: Some(threshold_h372),
                triggers: sum_h372 >= threshold_h372,
            });
        }
    }

    // Summation: H373
    let sum_h373: Decimal = resolved
        .iter()
        .filter(|r| !r.below_detection && r.clp.has_h_code("H373"))
        .map(|r| r.concentration_pct)
        .sum();

    if sum_h373 >= threshold_h373 {
        triggered = true;
    }

    for r in resolved {
        if r.below_detection {
            continue;
        }
        if r.clp.has_h_code("H373") {
            contributions.push(HpSubstanceContribution {
                substance: r.row.normalized_name.clone(),
                compound: r.compound_name.clone(),
                h_code: "H373".to_string(),
                concentration_pct: r.concentration_pct,
                threshold_pct: Some(threshold_h373),
                triggers: sum_h373 >= threshold_h373,
            });
        }
    }

    let reason = if triggered {
        format!(
            "STOT triggered (H372 sum: {:.4}% >= {}%, H373 sum: {:.4}% >= {}%)",
            sum_h372, threshold_h372, sum_h373, threshold_h373,
        )
    } else {
        format!(
            "STOT not triggered (H372 sum: {:.4}%, H373 sum: {:.4}%)",
            sum_h372, sum_h373,
        )
    };

    HpCriterionDetail {
        hp_id: "HP5".to_string(),
        hp_name: "STOT SE/RE".to_string(),
        triggered,
        reason,
        contributions,
    }
}

// ---------------------------------------------------------------------------
// HP6: Acute Toxicity (summation per H-code per route)
// H300 sum ≥ 0.1%, H301 sum ≥ 5%, H302 sum ≥ 25%
// H310 sum ≥ 0.1%, H311 sum ≥ 5%, H312 sum ≥ 25%
// H330 sum ≥ 0.1%, H331 sum ≥ 5%, H332 sum ≥ 25%
// ---------------------------------------------------------------------------

fn evaluate_hp6(resolved: &[ResolvedSubstance<'_>]) -> HpCriterionDetail {
    let checks: &[(&str, &str)] = &[
        ("H300", "0.1"),
        ("H301", "5.0"),
        ("H302", "25.0"),
        ("H310", "0.1"),
        ("H311", "5.0"),
        ("H312", "25.0"),
        ("H330", "0.1"),
        ("H331", "5.0"),
        ("H332", "25.0"),
    ];

    let mut contributions = Vec::new();
    let mut triggered = false;
    let mut trigger_details = Vec::new();

    for &(h_code, threshold_str) in checks {
        let threshold = dec_const(threshold_str);

        let sum: Decimal = resolved
            .iter()
            .filter(|r| !r.below_detection && r.clp.has_h_code(h_code))
            .map(|r| r.concentration_pct)
            .sum();

        let code_triggered = sum >= threshold;
        if code_triggered {
            triggered = true;
            trigger_details.push(format!("{} sum {:.4}% >= {}%", h_code, sum, threshold));
        }

        for r in resolved {
            if r.below_detection {
                continue;
            }
            if r.clp.has_h_code(h_code) {
                contributions.push(HpSubstanceContribution {
                    substance: r.row.normalized_name.clone(),
                    compound: r.compound_name.clone(),
                    h_code: h_code.to_string(),
                    concentration_pct: r.concentration_pct,
                    threshold_pct: Some(threshold),
                    triggers: code_triggered,
                });
            }
        }
    }

    let reason = if triggered {
        format!("Acute toxicity triggered: {}", trigger_details.join("; "))
    } else {
        "No acute toxicity summation thresholds exceeded".to_string()
    };

    HpCriterionDetail {
        hp_id: "HP6".to_string(),
        hp_name: "Acute Toxicity".to_string(),
        triggered,
        reason,
        contributions,
    }
}

// ---------------------------------------------------------------------------
// HP4: Irritant
// Summation: H315 sum ≥ 20%, H319 sum ≥ 20%
// ---------------------------------------------------------------------------

fn evaluate_hp4(resolved: &[ResolvedSubstance<'_>]) -> HpCriterionDetail {
    let checks: &[(&str, &str)] = &[("H315", "20.0"), ("H319", "20.0")];

    let mut contributions = Vec::new();
    let mut triggered = false;

    for &(h_code, threshold_str) in checks {
        let threshold = dec_const(threshold_str);

        let sum: Decimal = resolved
            .iter()
            .filter(|r| !r.below_detection && r.clp.has_h_code(h_code))
            .map(|r| r.concentration_pct)
            .sum();

        let code_triggered = sum >= threshold;
        if code_triggered {
            triggered = true;
        }

        for r in resolved {
            if r.below_detection {
                continue;
            }
            if r.clp.has_h_code(h_code) {
                contributions.push(HpSubstanceContribution {
                    substance: r.row.normalized_name.clone(),
                    compound: r.compound_name.clone(),
                    h_code: h_code.to_string(),
                    concentration_pct: r.concentration_pct,
                    threshold_pct: Some(threshold),
                    triggers: code_triggered,
                });
            }
        }
    }

    let reason = if triggered {
        "Irritant summation threshold exceeded".to_string()
    } else {
        "Irritant summation thresholds not exceeded".to_string()
    };

    HpCriterionDetail {
        hp_id: "HP4".to_string(),
        hp_name: "Irritant".to_string(),
        triggered,
        reason,
        contributions,
    }
}

// ---------------------------------------------------------------------------
// HP8: Corrosive
// Summation: H314 sum ≥ 5%
// ---------------------------------------------------------------------------

fn evaluate_hp8(resolved: &[ResolvedSubstance<'_>]) -> HpCriterionDetail {
    let threshold = dec_const("5.0");

    let sum: Decimal = resolved
        .iter()
        .filter(|r| !r.below_detection && r.clp.has_h_code("H314"))
        .map(|r| r.concentration_pct)
        .sum();

    let triggered = sum >= threshold;

    let contributions: Vec<HpSubstanceContribution> = resolved
        .iter()
        .filter(|r| !r.below_detection && r.clp.has_h_code("H314"))
        .map(|r| HpSubstanceContribution {
            substance: r.row.normalized_name.clone(),
            compound: r.compound_name.clone(),
            h_code: "H314".to_string(),
            concentration_pct: r.concentration_pct,
            threshold_pct: Some(threshold),
            triggers: triggered,
        })
        .collect();

    let reason = if triggered {
        format!("Corrosive: H314 sum {:.4}% >= {}%", sum, threshold)
    } else {
        format!(
            "Corrosive not triggered: H314 sum {:.4}% < {}%",
            sum, threshold
        )
    };

    HpCriterionDetail {
        hp_id: "HP8".to_string(),
        hp_name: "Corrosive".to_string(),
        triggered,
        reason,
        contributions,
    }
}

// ---------------------------------------------------------------------------
// HP13: Sensitising
// Individual limits: H317 ≥ 10%, H334 ≥ 10%
// ---------------------------------------------------------------------------

fn evaluate_hp13(resolved: &[ResolvedSubstance<'_>]) -> HpCriterionDetail {
    let threshold = dec_const("10.0");
    let mut contributions = Vec::new();
    let mut triggered = false;

    for h_code in &["H317", "H334"] {
        for r in resolved {
            if r.below_detection {
                continue;
            }
            if r.clp.has_h_code(h_code) {
                let triggers = r.concentration_pct >= threshold;
                if triggers {
                    triggered = true;
                }
                contributions.push(HpSubstanceContribution {
                    substance: r.row.normalized_name.clone(),
                    compound: r.compound_name.clone(),
                    h_code: h_code.to_string(),
                    concentration_pct: r.concentration_pct,
                    threshold_pct: Some(threshold),
                    triggers,
                });
            }
        }
    }

    let reason = if triggered {
        "Sensitising threshold exceeded".to_string()
    } else {
        "Sensitising thresholds not exceeded".to_string()
    };

    HpCriterionDetail {
        hp_id: "HP13".to_string(),
        hp_name: "Sensitising".to_string(),
        triggered,
        reason,
        contributions,
    }
}

// ---------------------------------------------------------------------------
// HP14: Ecotoxic (Regulation 2017/997)
// 4 parallel checks, any one triggering = FA:
//   1. Σ(c_i × M_acute) for H400 ≥ 25%
//   2. 100 × Σ(c_i × M_chronic) for H410 ≥ 25%
//   3. 10 × Σ(c_i × M_chronic) for H410 + Σ(c_i) for H411 ≥ 2.5%  (no H411 in our DB)
//   4. Σ(c_i) for H410+H411+H412+H413 ≥ 25% (with multipliers)    (simplified)
// ---------------------------------------------------------------------------

fn evaluate_hp14(resolved: &[ResolvedSubstance<'_>]) -> HpCriterionDetail {
    let threshold_check1 = dec_const("25.0");
    let threshold_check2 = dec_const("25.0");
    let _threshold_check3 = dec_const("2.5");
    let _threshold_check4 = dec_const("25.0");

    let hundred = dec_const("100");

    // Check 1: Σ(c_i × M_acute) for H400 substances ≥ 25%
    let sum_check1: Decimal = resolved
        .iter()
        .filter(|r| !r.below_detection && r.clp.has_h_code("H400"))
        .map(|r| {
            let m = r.clp.m_factors.acute.unwrap_or(ONE);
            r.concentration_pct * m
        })
        .sum();

    let check1_triggered = sum_check1 >= threshold_check1;

    // Check 2: 100 × Σ(c_i × M_chronic) for H410 substances ≥ 25%
    let sum_check2_raw: Decimal = resolved
        .iter()
        .filter(|r| !r.below_detection && r.clp.has_h_code("H410"))
        .map(|r| {
            let m = r.clp.m_factors.chronic.unwrap_or(ONE);
            r.concentration_pct * m
        })
        .sum();
    let sum_check2 = hundred * sum_check2_raw;

    let check2_triggered = sum_check2 >= threshold_check2;

    let triggered = check1_triggered || check2_triggered;

    // Build contributions for H400/H410 substances
    let mut contributions = Vec::new();
    for r in resolved {
        if r.below_detection {
            continue;
        }
        if r.clp.has_h_code("H400") {
            let m = r.clp.m_factors.acute.unwrap_or(ONE);
            contributions.push(HpSubstanceContribution {
                substance: r.row.normalized_name.clone(),
                compound: r.compound_name.clone(),
                h_code: "H400".to_string(),
                concentration_pct: r.concentration_pct * m,
                threshold_pct: Some(threshold_check1),
                triggers: check1_triggered,
            });
        }
        if r.clp.has_h_code("H410") {
            let m = r.clp.m_factors.chronic.unwrap_or(ONE);
            contributions.push(HpSubstanceContribution {
                substance: r.row.normalized_name.clone(),
                compound: r.compound_name.clone(),
                h_code: "H410".to_string(),
                concentration_pct: hundred * r.concentration_pct * m,
                threshold_pct: Some(threshold_check2),
                triggers: check2_triggered,
            });
        }
    }

    let reason = if triggered {
        let mut parts = Vec::new();
        if check1_triggered {
            parts.push(format!(
                "H400×M(ac) sum: {:.4}% >= {}%",
                sum_check1, threshold_check1
            ));
        }
        if check2_triggered {
            parts.push(format!(
                "100×H410×M(ch) sum: {:.4}% >= {}%",
                sum_check2, threshold_check2
            ));
        }
        format!("Ecotoxic triggered: {}", parts.join("; "))
    } else {
        format!(
            "Ecotoxic not triggered (H400×M sum: {:.4}%, 100×H410×M sum: {:.4}%)",
            sum_check1, sum_check2
        )
    };

    HpCriterionDetail {
        hp_id: "HP14".to_string(),
        hp_name: "Ecotoxic".to_string(),
        triggered,
        reason,
        contributions,
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Parse a decimal constant from a string.
fn dec_const(s: &str) -> Decimal {
    s.parse::<Decimal>().expect("valid decimal constant")
}

/// Get SCL for reproductive toxicity if present.
fn get_scl_for_repr(clp: &ClpSubstance, category: &str) -> Option<Decimal> {
    let key = format!("Repr.{}", category);
    clp.scls.get(&key).copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AnalysisReport, AnalysisRow, AnalysisValue, ReportHeader, Unit};
    use rust_decimal_macros::dec;

    fn row(name: &str, norm: &str, val: AnalysisValue) -> AnalysisRow {
        AnalysisRow {
            raw_name: name.into(),
            normalized_name: norm.into(),
            value: val,
            unit: Unit::MgPerKgTs,
        }
    }

    fn report(rows: Vec<AnalysisRow>) -> AnalysisReport {
        AnalysisReport {
            header: ReportHeader::default(),
            rows,
        }
    }

    #[test]
    fn test_hp7_arsenic_triggers() {
        // Arsenik 1200 mg/kg × 1.32 = 1584 mg/kg → 0.1584% >= 0.1% → H350 triggers
        let r = report(vec![row(
            "Arsenik (As)",
            "arsenik",
            AnalysisValue::Measured(dec!(1200)),
        )]);
        let result = classify_hp(&r);
        assert_eq!(result.overall_category, "FA");

        let hp7 = result
            .hp_details
            .as_ref()
            .unwrap()
            .criteria_results
            .iter()
            .find(|c| c.hp_id == "HP7")
            .unwrap();
        assert!(hp7.triggered);
    }

    #[test]
    fn test_hp7_arsenic_below_threshold() {
        // Arsenik 10 mg/kg × 1.32 = 13.2 mg/kg → 0.00132% < 0.1% → doesn't trigger
        let r = report(vec![row(
            "Arsenik (As)",
            "arsenik",
            AnalysisValue::Measured(dec!(10)),
        )]);
        let result = classify_hp(&r);

        let hp7 = result
            .hp_details
            .as_ref()
            .unwrap()
            .criteria_results
            .iter()
            .find(|c| c.hp_id == "HP7")
            .unwrap();
        assert!(!hp7.triggered);
    }

    #[test]
    fn test_hp10_lead_scl() {
        // Bly has SCL Repr.1A at 0.03% instead of GCL 0.3%
        // Bly 300 mg/kg × 1.00 = 300 → 0.03% = exactly 0.03% → triggers
        let r = report(vec![row(
            "Bly (Pb)",
            "bly",
            AnalysisValue::Measured(dec!(300)),
        )]);
        let result = classify_hp(&r);

        let hp10 = result
            .hp_details
            .as_ref()
            .unwrap()
            .criteria_results
            .iter()
            .find(|c| c.hp_id == "HP10")
            .unwrap();
        assert!(hp10.triggered);
    }

    #[test]
    fn test_hp10_lead_below_scl() {
        // Bly 200 mg/kg × 1.00 = 200 → 0.02% < 0.03% → doesn't trigger
        let r = report(vec![row(
            "Bly (Pb)",
            "bly",
            AnalysisValue::Measured(dec!(200)),
        )]);
        let result = classify_hp(&r);

        let hp10 = result
            .hp_details
            .as_ref()
            .unwrap()
            .criteria_results
            .iter()
            .find(|c| c.hp_id == "HP10")
            .unwrap();
        assert!(!hp10.triggered);
    }

    #[test]
    fn test_hp14_copper_m_factor() {
        // Koppar 500 mg/kg × 1.13 = 565 → 0.0565% w/w
        // H400: 0.0565 × M(ac)=100 = 5.65%
        // With only copper, check1 sum = 5.65% < 25% → doesn't trigger alone
        let r = report(vec![row(
            "Koppar (Cu)",
            "koppar",
            AnalysisValue::Measured(dec!(500)),
        )]);
        let result = classify_hp(&r);

        let hp14 = result
            .hp_details
            .as_ref()
            .unwrap()
            .criteria_results
            .iter()
            .find(|c| c.hp_id == "HP14")
            .unwrap();
        assert!(!hp14.triggered);
    }

    #[test]
    fn test_hp14_high_copper_triggers() {
        // Koppar 5000 mg/kg × 1.13 = 5650 → 0.565%
        // H400: 0.565 × M(ac)=100 = 56.5% >= 25% → triggers
        let r = report(vec![row(
            "Koppar (Cu)",
            "koppar",
            AnalysisValue::Measured(dec!(5000)),
        )]);
        let result = classify_hp(&r);

        let hp14 = result
            .hp_details
            .as_ref()
            .unwrap()
            .criteria_results
            .iter()
            .find(|c| c.hp_id == "HP14")
            .unwrap();
        assert!(hp14.triggered);
    }

    #[test]
    fn test_clean_soil_icke_fa() {
        // Low concentrations — nothing should trigger
        let r = report(vec![
            row("Arsenik (As)", "arsenik", AnalysisValue::Measured(dec!(5))),
            row("Bly (Pb)", "bly", AnalysisValue::Measured(dec!(20))),
            row(
                "Kadmium (Cd)",
                "kadmium",
                AnalysisValue::Measured(dec!(0.5)),
            ),
            row("Koppar (Cu)", "koppar", AnalysisValue::Measured(dec!(30))),
            row("Zink (Zn)", "zink", AnalysisValue::Measured(dec!(80))),
        ]);
        let result = classify_hp(&r);
        assert_eq!(result.overall_category, "Icke FA");
        assert!(result
            .hp_details
            .as_ref()
            .unwrap()
            .criteria_results
            .iter()
            .all(|c| !c.triggered));
    }

    #[test]
    fn test_below_detection_does_not_trigger() {
        // Even high detection limits should not trigger HP criteria
        let r = report(vec![row(
            "Arsenik (As)",
            "arsenik",
            AnalysisValue::BelowDetection(dec!(2000)),
        )]);
        let result = classify_hp(&r);
        assert_eq!(result.overall_category, "Icke FA");
    }

    #[test]
    fn test_hp11_chromium_mutagenic() {
        // Krom 1000 mg/kg × 1.92 = 1920 → 0.192% >= 0.1% → H340 triggers
        let r = report(vec![row(
            "Krom total",
            "krom_total",
            AnalysisValue::Measured(dec!(1000)),
        )]);
        let result = classify_hp(&r);

        let hp11 = result
            .hp_details
            .as_ref()
            .unwrap()
            .criteria_results
            .iter()
            .find(|c| c.hp_id == "HP11")
            .unwrap();
        assert!(hp11.triggered);
    }

    #[test]
    fn test_hp5_stot_mercury_chromium() {
        // Both HgCl2 and CrO3 have H372
        // Hg 200 mg/kg × 1.35 = 270 → 0.027%
        // Cr 3000 mg/kg × 1.92 = 5760 → 0.576%
        // Sum H372 = 0.027 + 0.576 = 0.603% < 1.0% → doesn't trigger
        let r = report(vec![
            row(
                "Kvicksilver (Hg)",
                "kvicksilver",
                AnalysisValue::Measured(dec!(200)),
            ),
            row(
                "Krom total",
                "krom_total",
                AnalysisValue::Measured(dec!(3000)),
            ),
        ]);
        let result = classify_hp(&r);

        let hp5 = result
            .hp_details
            .as_ref()
            .unwrap()
            .criteria_results
            .iter()
            .find(|c| c.hp_id == "HP5")
            .unwrap();
        assert!(!hp5.triggered);
    }

    #[test]
    fn test_hp5_stot_triggers() {
        // Hg 1000 mg/kg × 1.35 = 1350 → 0.135%
        // Cr 6000 mg/kg × 1.92 = 11520 → 1.152%
        // Cd 500 mg/kg × 1.14 = 570 → 0.057%
        // V2O5 has H372 too: V 2000 mg/kg × 1.78 = 3560 → 0.356%
        // Sum H372 = 0.135 + 1.152 + 0.057 + 0.356 = 1.700% >= 1.0% → triggers
        let r = report(vec![
            row(
                "Kvicksilver (Hg)",
                "kvicksilver",
                AnalysisValue::Measured(dec!(1000)),
            ),
            row(
                "Krom total",
                "krom_total",
                AnalysisValue::Measured(dec!(6000)),
            ),
            row(
                "Kadmium (Cd)",
                "kadmium",
                AnalysisValue::Measured(dec!(500)),
            ),
            row(
                "Vanadin (V)",
                "vanadin",
                AnalysisValue::Measured(dec!(2000)),
            ),
        ]);
        let result = classify_hp(&r);

        let hp5 = result
            .hp_details
            .as_ref()
            .unwrap()
            .criteria_results
            .iter()
            .find(|c| c.hp_id == "HP5")
            .unwrap();
        assert!(hp5.triggered);
    }

    #[test]
    fn test_overall_fa_with_multiple_triggers() {
        // Very contaminated soil: triggers HP7 (arsenic H350) and HP14 (copper H400 M=100)
        let r = report(vec![
            row(
                "Arsenik (As)",
                "arsenik",
                AnalysisValue::Measured(dec!(1500)),
            ),
            row("Koppar (Cu)", "koppar", AnalysisValue::Measured(dec!(5000))),
        ]);
        let result = classify_hp(&r);
        assert_eq!(result.overall_category, "FA");
        assert!(result.overall_reason.contains("HP7"));
        assert!(result.overall_reason.contains("HP14"));
    }

    #[test]
    fn test_naftalen_hp7_carc2() {
        // Naftalen has H351 (Carc. 2), threshold 1.0%
        // 10000 mg/kg = 1.0% → triggers
        let r = report(vec![row(
            "Naftalen",
            "naftalen",
            AnalysisValue::Measured(dec!(10000)),
        )]);
        let result = classify_hp(&r);

        let hp7 = result
            .hp_details
            .as_ref()
            .unwrap()
            .criteria_results
            .iter()
            .find(|c| c.hp_id == "HP7")
            .unwrap();
        assert!(hp7.triggered);
    }

    #[test]
    fn test_bap_hp11_mutagenic() {
        // BaP has H340 (Muta. 1B), threshold 0.1%
        // 1000 mg/kg = 0.1% → triggers
        let r = report(vec![row(
            "Benso(a)pyren",
            "benso_a_pyren",
            AnalysisValue::Measured(dec!(1000)),
        )]);
        let result = classify_hp(&r);

        let hp11 = result
            .hp_details
            .as_ref()
            .unwrap()
            .criteria_results
            .iter()
            .find(|c| c.hp_id == "HP11")
            .unwrap();
        assert!(hp11.triggered);
    }
}
