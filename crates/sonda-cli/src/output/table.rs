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
                        println!(
                            "    {} -> {}{}  ({} {})",
                            sr.raw_name, sr.category, uncertain_marker, sr.value, sr.unit
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
