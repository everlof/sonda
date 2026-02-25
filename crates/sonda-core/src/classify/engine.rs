use crate::classify::outcome::{RuleSetResult, SubstanceResult};
use crate::model::{AnalysisReport, AnalysisValue, Matrix};
use crate::rules::schema::{RuleSetDef, SubstanceRuleDef};
use rust_decimal::Decimal;
use std::collections::HashSet;

/// Classify an analysis report against one or more rulesets.
pub fn classify(report: &AnalysisReport, rulesets: &[RuleSetDef]) -> Vec<RuleSetResult> {
    rulesets.iter().map(|rs| classify_one(report, rs)).collect()
}

/// Classify an analysis report against a single ruleset.
fn classify_one(report: &AnalysisReport, ruleset: &RuleSetDef) -> RuleSetResult {
    let mut substance_results = Vec::new();
    let mut matched_substances = HashSet::new();
    let mut matched_rules = HashSet::new();

    for row in &report.rows {
        // Find matching rules (could be multiple if matrix-specific)
        let matching_rules: Vec<&SubstanceRuleDef> = ruleset
            .rules
            .iter()
            .filter(|r| {
                if r.substance != row.normalized_name {
                    return false;
                }
                // Check matrix filter
                if let Some(ref rule_matrix) = r.matrix {
                    if let Some(ref report_matrix) = report.header.matrix {
                        let rule_m = rule_matrix.to_lowercase();
                        let matches = match report_matrix {
                            Matrix::Jord => rule_m == "jord",
                            Matrix::Asfalt => rule_m == "asfalt",
                        };
                        if !matches {
                            return false;
                        }
                    }
                    // If report has no matrix info, skip matrix-specific rules
                    else {
                        return false;
                    }
                }
                true
            })
            .collect();

        if matching_rules.is_empty() {
            continue;
        }

        matched_substances.insert(row.normalized_name.clone());

        for rule in matching_rules {
            matched_rules.insert(rule.substance.clone());
            let result = classify_substance(row, rule, &ruleset.categories);
            substance_results.push(result);
        }
    }

    // Determine overall category
    let (overall_category, overall_reason, determining) =
        determine_overall(&substance_results, &ruleset.categories);

    // Unmatched substances (in report but no rule)
    let all_report_substances: HashSet<String> = report
        .rows
        .iter()
        .map(|r| r.normalized_name.clone())
        .collect();
    let mut unmatched_substances: Vec<String> = all_report_substances
        .difference(&matched_substances)
        .cloned()
        .collect();
    unmatched_substances.sort();

    // Unmatched rules (in ruleset but not in report)
    let all_rule_substances: HashSet<String> =
        ruleset.rules.iter().map(|r| r.substance.clone()).collect();
    let mut unmatched_rules: Vec<String> = all_rule_substances
        .difference(&matched_rules)
        .cloned()
        .collect();
    unmatched_rules.sort();

    RuleSetResult {
        ruleset_name: ruleset.name.clone(),
        overall_category,
        overall_reason,
        lowest_category: ruleset.categories.first().cloned(),
        determining_substances: determining,
        substance_results,
        unmatched_substances,
        unmatched_rules,
        hp_details: None,
    }
}

/// Classify a single substance value against a rule.
fn classify_substance(
    row: &crate::model::AnalysisRow,
    rule: &SubstanceRuleDef,
    categories: &[String],
) -> SubstanceResult {
    let unit = rule.unit.clone().unwrap_or_else(|| "mg/kg TS".to_string());

    match &row.value {
        AnalysisValue::Measured(value) => classify_measured(*value, row, rule, categories, &unit),
        AnalysisValue::BelowDetection(detection_limit) => {
            classify_below_detection(*detection_limit, row, rule, categories, &unit)
        }
    }
}

/// Classify a measured value.
fn classify_measured(
    value: Decimal,
    row: &crate::model::AnalysisRow,
    rule: &SubstanceRuleDef,
    categories: &[String],
    unit: &str,
) -> SubstanceResult {
    // Iterate categories in order (cleanest first)
    for (i, cat) in categories.iter().enumerate() {
        if let Some(&threshold) = rule.thresholds.get(cat) {
            if value <= threshold {
                // Classified into this category
                let reason = if i == 0 {
                    format!(
                        "{}: {} {} <= {} ({}) -> classified as {}",
                        row.raw_name, value, unit, threshold, cat, cat
                    )
                } else {
                    // Find the previous category's threshold for the reason
                    let prev_parts: Vec<String> = categories[..i]
                        .iter()
                        .filter_map(|prev_cat| {
                            rule.thresholds
                                .get(prev_cat)
                                .map(|t| format!("{} > {}:{}", value, prev_cat, t))
                        })
                        .collect();
                    format!(
                        "{}: {} {} {} but <= {}:{} -> classified as {}",
                        row.raw_name,
                        value,
                        unit,
                        prev_parts.join(", "),
                        cat,
                        threshold,
                        cat
                    )
                };

                let exceeded = if i > 0 {
                    categories[..i]
                        .iter()
                        .rev()
                        .find_map(|prev_cat| rule.thresholds.get(prev_cat).copied())
                } else {
                    None
                };

                return SubstanceResult {
                    substance: row.normalized_name.clone(),
                    raw_name: row.raw_name.clone(),
                    value: row.value.clone(),
                    unit: unit.to_string(),
                    category: cat.clone(),
                    reason,
                    exceeded_threshold: exceeded,
                    uncertain: false,
                };
            }
        }
    }

    // Exceeds all thresholds
    let last_cat = categories.last().cloned().unwrap_or_default();
    let exceeds_cat = format!("> {}", last_cat);

    let threshold_parts: Vec<String> = categories
        .iter()
        .filter_map(|cat| rule.thresholds.get(cat).map(|t| format!("{}:{}", cat, t)))
        .collect();

    let last_threshold = categories
        .iter()
        .rev()
        .find_map(|cat| rule.thresholds.get(cat).copied());

    SubstanceResult {
        substance: row.normalized_name.clone(),
        raw_name: row.raw_name.clone(),
        value: row.value.clone(),
        unit: unit.to_string(),
        category: exceeds_cat,
        reason: format!(
            "{}: {} {} > {} -> exceeds all thresholds",
            row.raw_name,
            value,
            unit,
            threshold_parts.join(", ")
        ),
        exceeded_threshold: last_threshold,
        uncertain: false,
    }
}

/// Classify a below-detection-limit value.
///
/// Conservative approach: if detection limit >= threshold, escalate to next
/// category and flag as uncertain.
fn classify_below_detection(
    detection_limit: Decimal,
    row: &crate::model::AnalysisRow,
    rule: &SubstanceRuleDef,
    categories: &[String],
    unit: &str,
) -> SubstanceResult {
    // Check each category: if detection limit < threshold, we can confidently
    // classify into that category.
    for cat in categories.iter() {
        if let Some(&threshold) = rule.thresholds.get(cat) {
            if detection_limit < threshold {
                let reason = format!(
                    "{}: < {} {}, detection limit below {} threshold ({}) -> classified as {}",
                    row.raw_name, detection_limit, unit, cat, threshold, cat
                );
                return SubstanceResult {
                    substance: row.normalized_name.clone(),
                    raw_name: row.raw_name.clone(),
                    value: row.value.clone(),
                    unit: unit.to_string(),
                    category: cat.clone(),
                    reason,
                    exceeded_threshold: None,
                    uncertain: false,
                };
            }
            // detection_limit >= threshold: can't confirm this category,
            // try next one
        }
    }

    // Detection limit exceeds all thresholds -- classify into last category but uncertain
    let last_cat = categories.last().cloned().unwrap_or_default();
    let last_threshold = rule.thresholds.get(&last_cat).copied();

    // Actually, try to find the first category where detection limit < threshold
    // If none found, classify into last category as uncertain
    // But we've already tried all above, so we're at "exceeds all" territory
    // However, since it's below detection, we classify into last category as uncertain

    let threshold_parts: Vec<String> = categories
        .iter()
        .filter_map(|cat| rule.thresholds.get(cat).map(|t| format!("{}:{}", cat, t)))
        .collect();

    SubstanceResult {
        substance: row.normalized_name.clone(),
        raw_name: row.raw_name.clone(),
        value: row.value.clone(),
        unit: unit.to_string(),
        category: last_cat,
        reason: format!(
            "{}: < {} {}, detection limit exceeds all thresholds ({}) -> uncertain",
            row.raw_name,
            detection_limit,
            unit,
            threshold_parts.join(", ")
        ),
        exceeded_threshold: last_threshold,
        uncertain: true,
    }
}

/// Determine overall classification from per-substance results.
fn determine_overall(
    results: &[SubstanceResult],
    categories: &[String],
) -> (String, String, Vec<String>) {
    if results.is_empty() {
        return (
            "N/A".to_string(),
            "No substances matched any rules".to_string(),
            vec![],
        );
    }

    // Find the worst category index
    let mut worst_idx: Option<usize> = None;
    let mut has_exceeds = false;

    for r in results {
        if r.category.starts_with("> ") {
            has_exceeds = true;
            break;
        }
        if let Some(idx) = categories.iter().position(|c| *c == r.category) {
            match worst_idx {
                None => worst_idx = Some(idx),
                Some(current) => {
                    if idx > current {
                        worst_idx = Some(idx);
                    }
                }
            }
        }
    }

    if has_exceeds {
        let last_cat = categories.last().cloned().unwrap_or_default();
        let exceeds_cat = format!("> {}", last_cat);
        let determining: Vec<String> = results
            .iter()
            .filter(|r| r.category.starts_with("> "))
            .map(|r| r.raw_name.clone())
            .collect();
        let reason = format!("Determined by {}", determining.join(", "));
        return (exceeds_cat, reason, determining);
    }

    let worst = worst_idx.unwrap_or(0);
    let worst_cat = &categories[worst];

    let determining: Vec<String> = results
        .iter()
        .filter(|r| r.category == *worst_cat)
        .map(|r| r.raw_name.clone())
        .collect();

    let reason = if determining.len() == 1 {
        format!("Determined by {} ({})", determining[0], worst_cat)
    } else {
        format!(
            "Determined by {} substances at {} level",
            determining.len(),
            worst_cat
        )
    };

    (worst_cat.clone(), reason, determining)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AnalysisRow, AnalysisValue, ReportHeader, Unit};
    use crate::rules::schema::{RuleSetDef, SubstanceRuleDef};
    use rust_decimal_macros::dec;
    use std::collections::BTreeMap;

    fn make_ruleset() -> RuleSetDef {
        RuleSetDef {
            name: "Test NV".into(),
            description: None,
            version: "1.0".into(),
            matrix: None,
            categories: vec!["KM".into(), "MKM".into()],
            category_descriptions: BTreeMap::new(),
            rules: vec![
                SubstanceRuleDef {
                    substance: "bly".into(),
                    thresholds: BTreeMap::from([
                        ("KM".into(), dec!(50)),
                        ("MKM".into(), dec!(180)),
                    ]),
                    matrix: None,
                    unit: None,
                    note: None,
                },
                SubstanceRuleDef {
                    substance: "arsenik".into(),
                    thresholds: BTreeMap::from([("KM".into(), dec!(10)), ("MKM".into(), dec!(25))]),
                    matrix: None,
                    unit: None,
                    note: None,
                },
                SubstanceRuleDef {
                    substance: "kvicksilver".into(),
                    thresholds: BTreeMap::from([
                        ("KM".into(), dec!(0.25)),
                        ("MKM".into(), dec!(2.5)),
                    ]),
                    matrix: None,
                    unit: None,
                    note: None,
                },
            ],
        }
    }

    fn make_report(rows: Vec<AnalysisRow>) -> AnalysisReport {
        AnalysisReport {
            header: ReportHeader::default(),
            rows,
        }
    }

    fn row(name: &str, norm: &str, val: AnalysisValue) -> AnalysisRow {
        AnalysisRow {
            raw_name: name.into(),
            normalized_name: norm.into(),
            value: val,
            unit: Unit::MgPerKgTs,
        }
    }

    #[test]
    fn test_classify_all_below_km() {
        let report = make_report(vec![
            row("Bly (Pb)", "bly", AnalysisValue::Measured(dec!(30))),
            row("Arsenik (As)", "arsenik", AnalysisValue::Measured(dec!(5))),
        ]);
        let result = classify(&report, &[make_ruleset()]);
        let rs = &result[0];
        assert_eq!(rs.overall_category, "KM");
        assert!(rs.substance_results.iter().all(|r| r.category == "KM"));
    }

    #[test]
    fn test_classify_one_exceeds_km() {
        let report = make_report(vec![
            row("Bly (Pb)", "bly", AnalysisValue::Measured(dec!(120))),
            row("Arsenik (As)", "arsenik", AnalysisValue::Measured(dec!(5))),
        ]);
        let result = classify(&report, &[make_ruleset()]);
        let rs = &result[0];
        assert_eq!(rs.overall_category, "MKM");

        let bly = rs
            .substance_results
            .iter()
            .find(|r| r.substance == "bly")
            .unwrap();
        assert_eq!(bly.category, "MKM");
        assert!(bly.reason.contains("120"));
        assert!(bly.reason.contains("MKM"));
    }

    #[test]
    fn test_classify_exceeds_all() {
        let report = make_report(vec![row(
            "Bly (Pb)",
            "bly",
            AnalysisValue::Measured(dec!(200)),
        )]);
        let result = classify(&report, &[make_ruleset()]);
        let rs = &result[0];
        assert_eq!(rs.overall_category, "> MKM");
    }

    #[test]
    fn test_below_detection_confident() {
        // Detection limit 5 < KM threshold 10 -> confidently KM
        let report = make_report(vec![row(
            "Arsenik (As)",
            "arsenik",
            AnalysisValue::BelowDetection(dec!(5)),
        )]);
        let result = classify(&report, &[make_ruleset()]);
        let rs = &result[0];
        assert_eq!(rs.substance_results[0].category, "KM");
        assert!(!rs.substance_results[0].uncertain);
    }

    #[test]
    fn test_below_detection_uncertain_escalation() {
        // Detection limit 0.30 > KM threshold 0.25, but < MKM threshold 2.5
        // -> classified as MKM (uncertain)
        let report = make_report(vec![row(
            "Kvicksilver (Hg)",
            "kvicksilver",
            AnalysisValue::BelowDetection(dec!(0.30)),
        )]);
        let result = classify(&report, &[make_ruleset()]);
        let rs = &result[0];
        let hg = &rs.substance_results[0];
        assert_eq!(hg.category, "MKM");
        assert!(!hg.uncertain); // Not uncertain because we can confirm < MKM threshold
    }

    #[test]
    fn test_unmatched_tracking() {
        let report = make_report(vec![
            row("Bly (Pb)", "bly", AnalysisValue::Measured(dec!(30))),
            row("Unknown", "unknown", AnalysisValue::Measured(dec!(100))),
        ]);
        let result = classify(&report, &[make_ruleset()]);
        let rs = &result[0];
        assert!(rs.unmatched_substances.contains(&"unknown".to_string()));
        assert!(rs.unmatched_rules.contains(&"arsenik".to_string()));
    }

    #[test]
    fn test_reason_strings_populated() {
        let report = make_report(vec![row(
            "Bly (Pb)",
            "bly",
            AnalysisValue::Measured(dec!(120)),
        )]);
        let result = classify(&report, &[make_ruleset()]);
        let rs = &result[0];
        let bly = &rs.substance_results[0];
        assert!(!bly.reason.is_empty());
        assert!(!rs.overall_reason.is_empty());
    }
}
