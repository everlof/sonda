pub mod classify;
pub mod clp;
pub mod error;
pub mod extraction;
pub mod model;
pub mod parsing;
pub mod rules;
pub mod trace;

use classify::outcome::{ClassificationResult, SampleResult};
use error::SondaError;
use extraction::PdfExtractor;
use model::{AnalysisReport, Matrix};
use rules::schema::RuleSetDef;

/// Options controlling which classification engines to run.
#[derive(Debug, Clone, Default)]
pub struct ClassifyOptions {
    /// Run HP-based hazardous waste (FA) classification.
    pub include_hp: bool,
}

/// Main API entry point: classify a PDF report against one or more rulesets.
///
/// Handles multi-sample PDFs by splitting and classifying each sample
/// independently. Filters rulesets by matrix per sample.
pub fn classify_pdf(
    pdf_bytes: &[u8],
    extractor: &dyn PdfExtractor,
    rulesets: &[RuleSetDef],
    options: &ClassifyOptions,
) -> Result<ClassificationResult, SondaError> {
    // Extract text from PDF
    let pages = extractor.extract_pages(pdf_bytes)?;

    // Parse into one or more reports (one per sample)
    let parsed = parsing::parse_reports(&pages)?;
    let reports = parsed.reports;
    let mut trace = trace::TraceBundle::default();

    // Check for supported lab (check the first report)
    if reports
        .first()
        .and_then(|r| r.header.lab.as_ref())
        .is_none()
    {
        return Err(SondaError::UnsupportedReport(
            "could not detect laboratory. Look for 'Eurofins' in the report header".into(),
        ));
    }

    // Classify each sample
    let mut samples = Vec::new();
    for report in &reports {
        let sample_result = classify_sample(report, rulesets, options)?;

        for (entry_idx, row) in report.rows.iter().enumerate() {
            trace.entries.push(trace::build_entry_trace(
                &sample_result.sample_id,
                entry_idx,
                row,
                &row.unit.to_string(),
                &pages,
            ));
        }

        for (rs_idx, rs) in sample_result.ruleset_results.iter().enumerate() {
            trace.decisions.extend(trace::build_ruleset_decisions(
                &sample_result.sample_id,
                rs_idx,
                rs,
            ));
        }

        samples.push(sample_result);
    }

    // Surface skipped lines as trace warnings (Info/Auto — diagnostic, not critical)
    for skip in parsed.skipped_lines {
        trace.warnings.push(trace::TraceWarning {
            sample_id: None,
            message: format!("Skipped line ({}): '{}'", skip.reason, skip.line_text),
            severity: trace::TraceSeverity::Info,
            visibility: trace::TraceVisibility::Auto,
        });
    }

    let warnings = parsed
        .warnings
        .into_iter()
        .map(|w| {
            let message = if let Some(ref id) = w.sample_id {
                format!(
                    "Skipped sample '{}' (section {}): {}",
                    id, w.section_index, w.reason
                )
            } else {
                format!("Skipped section {}: {}", w.section_index, w.reason)
            };
            trace.warnings.push(trace::TraceWarning {
                sample_id: w.sample_id.clone(),
                message: message.clone(),
                severity: trace::TraceSeverity::Important,
                visibility: trace::TraceVisibility::Always,
            });
            classify::outcome::ParseWarning {
                sample_id: w.sample_id,
                message,
            }
        })
        .collect();

    Ok(ClassificationResult {
        samples,
        warnings,
        trace,
    })
}

/// Classify a single sample report against applicable rulesets.
fn classify_sample(
    report: &AnalysisReport,
    rulesets: &[RuleSetDef],
    options: &ClassifyOptions,
) -> Result<SampleResult, SondaError> {
    // Build sample ID from header
    let sample_id = report
        .header
        .sample_id
        .clone()
        .or_else(|| report.header.lab_report_id.clone())
        .unwrap_or_else(|| "unknown".into());

    let mut ruleset_results = Vec::new();

    // Run threshold-based classification if we have applicable rulesets
    if !rulesets.is_empty() {
        let applicable: Vec<&RuleSetDef> = rulesets
            .iter()
            .filter(|rs| match rs.matrix.as_deref() {
                None => true,
                Some(ruleset_matrix) => match &report.header.matrix {
                    Some(report_matrix) => {
                        let rm = ruleset_matrix.to_lowercase();
                        match report_matrix {
                            Matrix::Jord => rm == "jord",
                            Matrix::Asfalt => rm == "asfalt",
                        }
                    }
                    None => false,
                },
            })
            .collect();

        if applicable.is_empty() && !options.include_hp {
            let matrix_str = report
                .header
                .matrix
                .as_ref()
                .map(|m| m.to_string())
                .unwrap_or_else(|| "unknown".into());
            return Err(SondaError::MatrixMismatch { matrix: matrix_str });
        }

        let applicable_owned: Vec<RuleSetDef> = applicable.into_iter().cloned().collect();
        let threshold_results = classify::classify(report, &applicable_owned);
        ruleset_results.extend(threshold_results);
    }

    // Run HP-based classification if requested
    if options.include_hp {
        let hp_result = classify::hp_engine::classify_hp(report);
        ruleset_results.push(hp_result);
    }

    if ruleset_results.is_empty() {
        let matrix_str = report
            .header
            .matrix
            .as_ref()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "unknown".into());
        return Err(SondaError::MatrixMismatch { matrix: matrix_str });
    }

    Ok(SampleResult {
        sample_id,
        ruleset_results,
    })
}
