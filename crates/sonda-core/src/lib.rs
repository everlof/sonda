pub mod classify;
pub mod error;
pub mod extraction;
pub mod model;
pub mod parsing;
pub mod rules;

use classify::outcome::{ClassificationResult, SampleResult};
use error::SondaError;
use extraction::PdfExtractor;
use model::{AnalysisReport, Matrix};
use rules::schema::RuleSetDef;

/// Main API entry point: classify a PDF report against one or more rulesets.
///
/// Handles multi-sample PDFs by splitting and classifying each sample
/// independently. Filters rulesets by matrix per sample.
pub fn classify_pdf(
    pdf_bytes: &[u8],
    extractor: &dyn PdfExtractor,
    rulesets: &[RuleSetDef],
) -> Result<ClassificationResult, SondaError> {
    // Extract text from PDF
    let pages = extractor.extract_pages(pdf_bytes)?;

    // Parse into one or more reports (one per sample)
    let reports = parsing::parse_reports(&pages)?;

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
        let sample_result = classify_sample(report, rulesets)?;
        samples.push(sample_result);
    }

    Ok(ClassificationResult { samples })
}

/// Classify a single sample report against applicable rulesets.
fn classify_sample(
    report: &AnalysisReport,
    rulesets: &[RuleSetDef],
) -> Result<SampleResult, SondaError> {
    // Build sample ID from header
    let sample_id = report
        .header
        .sample_id
        .clone()
        .or_else(|| report.header.lab_report_id.clone())
        .unwrap_or_else(|| "unknown".into());

    // Filter rulesets by matrix
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

    if applicable.is_empty() {
        let matrix_str = report
            .header
            .matrix
            .as_ref()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "unknown".into());
        return Err(SondaError::MatrixMismatch { matrix: matrix_str });
    }

    let applicable_owned: Vec<RuleSetDef> = applicable.into_iter().cloned().collect();
    let result = classify::classify(report, &applicable_owned);

    Ok(SampleResult {
        sample_id,
        ruleset_results: result,
    })
}
