use sonda_core::classify::outcome::ClassificationResult;
use sonda_core::parsing::ParsedReports;

/// Format parsed reports as a human-readable table.
pub fn format_parsed(parsed: &ParsedReports) -> String {
    let mut out = String::new();

    let multi_sample = parsed.reports.len() > 1;

    for (i, report) in parsed.reports.iter().enumerate() {
        if multi_sample {
            if i > 0 {
                out.push('\n');
            }
            let sample_id = report
                .header
                .sample_id
                .as_deref()
                .or(report.header.lab_report_id.as_deref())
                .unwrap_or("unknown");
            out.push_str(&format!("--- Sample: {} ---\n\n", sample_id));
        }

        // Header info
        if let Some(ref lab) = report.header.lab {
            out.push_str(&format!("  Lab:       {}\n", lab));
        }
        if let Some(ref id) = report.header.lab_report_id {
            out.push_str(&format!("  Report ID: {}\n", id));
        }
        if let Some(ref id) = report.header.sample_id {
            out.push_str(&format!("  Sample ID: {}\n", id));
        }
        if let Some(ref matrix) = report.header.matrix {
            out.push_str(&format!("  Matrix:    {}\n", matrix));
        }
        if let Some(ref date) = report.header.date {
            out.push_str(&format!("  Date:      {}\n", date));
        }
        out.push('\n');

        if report.rows.is_empty() {
            out.push_str("  (no substances parsed)\n");
            continue;
        }

        // Column widths
        let max_raw = report
            .rows
            .iter()
            .map(|r| r.raw_name.len())
            .max()
            .unwrap_or(10)
            .max(8);
        let max_norm = report
            .rows
            .iter()
            .map(|r| r.normalized_name.len())
            .max()
            .unwrap_or(10)
            .max(10);
        let max_val = report
            .rows
            .iter()
            .map(|r| format!("{}", r.value).len())
            .max()
            .unwrap_or(8)
            .max(5);

        // Header
        out.push_str(&format!(
            "  {:<raw_w$}  {:<norm_w$}  {:>val_w$}  Unit\n",
            "Raw name",
            "Normalized",
            "Value",
            raw_w = max_raw,
            norm_w = max_norm,
            val_w = max_val,
        ));
        out.push_str(&format!(
            "  {}\n",
            "-".repeat(max_raw + max_norm + max_val + 14)
        ));

        for row in &report.rows {
            let val_str = format!("{}", row.value);
            out.push_str(&format!(
                "  {:<raw_w$}  {:<norm_w$}  {:>val_w$}  {}\n",
                row.raw_name,
                row.normalized_name,
                val_str,
                row.unit,
                raw_w = max_raw,
                norm_w = max_norm,
                val_w = max_val,
            ));
        }
    }

    // Warnings
    if !parsed.warnings.is_empty() {
        out.push_str("\nWarnings:\n");
        for w in &parsed.warnings {
            out.push_str(&format!("  - section {}: {}\n", w.section_index, w.reason));
        }
    }

    // Skipped lines
    if !parsed.skipped_lines.is_empty() {
        out.push_str(&format!(
            "\nSkipped lines ({}):\n",
            parsed.skipped_lines.len()
        ));
        for sl in &parsed.skipped_lines {
            out.push_str(&format!("  - [{}] {}\n", sl.reason, sl.line_text));
        }
    }

    out
}

pub fn print(result: &ClassificationResult, show_all: bool, verbose: bool) {
    if !result.warnings.is_empty() {
        println!("Warnings:\n");
        for w in &result.warnings {
            println!("  - {}", w.message);
        }
        println!();
    }

    let multi_sample = result.samples.len() > 1;

    for (i, sample) in result.samples.iter().enumerate() {
        if multi_sample {
            if i > 0 {
                println!();
            }
            println!("--- Sample: {} ---\n", sample.sample_id);
        }

        for rs_result in &sample.ruleset_results {
            println!("=== {} ===\n", rs_result.ruleset_name);

            // HP-based output
            if let Some(ref hp) = rs_result.hp_details {
                print_hp_result(rs_result, hp, verbose);
                continue;
            }

            // Threshold-based output (existing logic)
            // Overall classification
            println!(
                "  Overall: {} ({})\n",
                rs_result.overall_category, rs_result.overall_reason
            );

            // Per-substance results
            if verbose || show_all {
                let results_to_show: Vec<_> = rs_result.substance_results.iter().collect();

                if !results_to_show.is_empty() {
                    let max_name = results_to_show
                        .iter()
                        .map(|r| r.raw_name.len())
                        .max()
                        .unwrap_or(10);

                    for sr in &results_to_show {
                        let uncertain_marker = if sr.uncertain { " (?)" } else { "" };
                        println!(
                            "  {:<width$}  {} {}  -> {}{}",
                            sr.raw_name,
                            sr.value,
                            sr.unit,
                            sr.category,
                            uncertain_marker,
                            width = max_name
                        );
                        if verbose {
                            println!("    {}", sr.reason);
                        }
                    }
                    println!();
                }
            }

            // Exceedances summary (non-verbose mode)
            if !verbose && !show_all {
                if let Some(cleanest) = rs_result.lowest_category.as_deref() {
                    if rs_result.overall_category != cleanest {
                        let determining: Vec<_> = rs_result
                            .substance_results
                            .iter()
                            .filter(|r| rs_result.determining_substances.contains(&r.raw_name))
                            .collect();

                        if !determining.is_empty() {
                            println!("  Determining substances:");
                            for sr in &determining {
                                let uncertain_marker = if sr.uncertain { " (?)" } else { "" };
                                let threshold_info = match sr.exceeded_threshold {
                                    Some(t) => format!("{} > {} {}", sr.value, t, sr.unit),
                                    None => format!("{} {}", sr.value, sr.unit),
                                };
                                println!(
                                    "    {} -> {}{}  ({})",
                                    sr.raw_name, sr.category, uncertain_marker, threshold_info
                                );
                            }
                            println!();
                        }
                    }
                }
            }

            // Unmatched info
            if verbose {
                if !rs_result.unmatched_rules.is_empty() {
                    println!(
                        "  Rules without matching report data: {}",
                        rs_result.unmatched_rules.join(", ")
                    );
                }
                if !rs_result.unmatched_substances.is_empty() {
                    println!(
                        "  Report substances without rules: {}",
                        rs_result.unmatched_substances.join(", ")
                    );
                }
                println!();
            }
        }
    }
}

fn print_hp_result(
    rs_result: &sonda_core::classify::outcome::RuleSetResult,
    hp: &sonda_core::classify::outcome::HpDetails,
    verbose: bool,
) {
    // Overall classification
    let triggered_ids: Vec<&str> = hp
        .criteria_results
        .iter()
        .filter(|c| c.triggered)
        .map(|c| c.hp_id.as_str())
        .collect();

    if hp.is_hazardous {
        println!(
            "  Overall: {} (triggered by {})\n",
            rs_result.overall_category,
            triggered_ids.join(", ")
        );
    } else {
        println!("  Overall: {}\n", rs_result.overall_category);
    }

    if verbose {
        // Verbose: show all criteria with details
        for cr in &hp.criteria_results {
            let status = if cr.triggered {
                "TRIGGERED"
            } else {
                "not triggered"
            };
            println!("  {} ({}): {}", cr.hp_id, cr.hp_name, status);

            if !cr.contributions.is_empty() {
                for c in &cr.contributions {
                    let trigger_marker = if c.triggers { " ***" } else { "" };
                    if let Some(threshold) = c.threshold_pct {
                        let comparison = if c.triggers { ">=" } else { "<" };
                        println!(
                            "    {} -> {} ({}): {:.4}% {} {}%{}",
                            c.substance,
                            c.compound,
                            c.h_code,
                            c.concentration_pct,
                            comparison,
                            threshold,
                            trigger_marker,
                        );
                    } else {
                        println!(
                            "    {} -> {} ({}): {:.4}%{}",
                            c.substance, c.compound, c.h_code, c.concentration_pct, trigger_marker,
                        );
                    }
                }
            }
            println!();
        }
    } else {
        // Non-verbose: only show triggered criteria
        if hp.is_hazardous {
            println!("  Triggered HP criteria:");
            for cr in &hp.criteria_results {
                if !cr.triggered {
                    continue;
                }
                // Show the triggering contributions
                let triggering: Vec<String> = cr
                    .contributions
                    .iter()
                    .filter(|c| c.triggers)
                    .map(|c| {
                        if let Some(threshold) = c.threshold_pct {
                            format!(
                                "{} ({}): {:.4}% >= {}%",
                                c.compound, c.h_code, c.concentration_pct, threshold
                            )
                        } else {
                            format!("{} ({}): {:.4}%", c.compound, c.h_code, c.concentration_pct)
                        }
                    })
                    .collect();

                let details = if triggering.is_empty() {
                    cr.reason.clone()
                } else {
                    triggering.join(", ")
                };

                println!("    {:<5} {:<25} {}", cr.hp_id, cr.hp_name, details);
            }
            println!();
        }
    }
}
