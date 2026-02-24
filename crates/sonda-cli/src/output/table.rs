use sonda_core::classify::outcome::ClassificationResult;

pub fn print(result: &ClassificationResult, show_all: bool, verbose: bool) {
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
                let results_to_show: Vec<_> = if show_all {
                    rs_result.substance_results.iter().collect()
                } else {
                    rs_result
                        .substance_results
                        .iter()
                        .filter(|r| {
                            r.category
                                != rs_result
                                    .substance_results
                                    .first()
                                    .map(|f| f.category.as_str())
                                    .unwrap_or("")
                        })
                        .collect()
                };

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
                let exceedances: Vec<_> = rs_result
                    .substance_results
                    .iter()
                    .filter(|r| r.category == rs_result.overall_category && r.category != "KM")
                    .collect();

                if !exceedances.is_empty() {
                    println!("  Determining substances:");
                    for sr in &exceedances {
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
