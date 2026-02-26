use sonda_core::extraction::pdftotext::PdftotextExtractor;
use sonda_core::model::AnalysisReport;
use sonda_core::rules::builtin;
use sonda_core::rules::schema::RuleSetDef;
use sonda_core::ClassifyOptions;
use std::path::PathBuf;

use crate::output;

pub fn run(
    input_file: PathBuf,
    rule_files: Vec<PathBuf>,
    presets: Vec<String>,
    output_format: &str,
    show_all: bool,
    verbose: bool,
) -> Result<(), sonda_core::error::SondaError> {
    // Load rulesets
    let mut rulesets: Vec<RuleSetDef> = Vec::new();
    let mut options = ClassifyOptions::default();

    // Default to all presets if no presets or custom rules specified.
    // The engine filters by matrix automatically.
    let effective_presets = if presets.is_empty() && rule_files.is_empty() {
        builtin::PRESETS.iter().map(|s| s.to_string()).collect()
    } else {
        presets
    };

    // Load presets (separating HP-based from threshold-based)
    for preset in &effective_presets {
        if builtin::is_hp_preset(preset) {
            options.include_hp = true;
        } else {
            let rs = builtin::load_preset(preset)?;
            rulesets.push(rs);
        }
    }

    // Load custom rule files
    for path in &rule_files {
        let rs = sonda_core::rules::load_ruleset(path)?;
        rulesets.push(rs);
    }

    if rulesets.is_empty() && !options.include_hp {
        return Err(sonda_core::error::SondaError::RulesetInvalid(
            "no rulesets specified".into(),
        ));
    }

    // Determine input type by extension
    let is_json = input_file
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    let result = if is_json {
        // Load pre-parsed reports from JSON
        let json_bytes = std::fs::read(&input_file)?;
        let reports: Vec<AnalysisReport> = serde_json::from_slice(&json_bytes)?;
        sonda_core::classify_reports(&reports, &rulesets, &options)?
    } else {
        // Parse and classify PDF
        let pdf_bytes = std::fs::read(&input_file)?;
        let extractor = PdftotextExtractor::new();
        sonda_core::classify_pdf(&pdf_bytes, &extractor, &rulesets, &options)?
    };

    // Output
    match output_format {
        "json" => output::json::print(&result)?,
        _ => output::table::print(&result, show_all, verbose),
    }

    Ok(())
}
