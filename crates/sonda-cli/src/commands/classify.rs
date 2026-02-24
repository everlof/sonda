use sonda_core::extraction::pdftotext::PdftotextExtractor;
use sonda_core::rules::builtin;
use sonda_core::rules::schema::RuleSetDef;
use std::path::PathBuf;

use crate::output;

pub fn run(
    pdf_file: PathBuf,
    rule_files: Vec<PathBuf>,
    presets: Vec<String>,
    output_format: &str,
    show_all: bool,
    verbose: bool,
) -> Result<(), sonda_core::error::SondaError> {
    // Load rulesets
    let mut rulesets: Vec<RuleSetDef> = Vec::new();

    // Default to all presets if no presets or custom rules specified.
    // The engine filters by matrix automatically.
    let effective_presets = if presets.is_empty() && rule_files.is_empty() {
        builtin::PRESETS.iter().map(|s| s.to_string()).collect()
    } else {
        presets
    };

    // Load presets
    for preset in &effective_presets {
        let rs = builtin::load_preset(preset)?;
        rulesets.push(rs);
    }

    // Load custom rule files
    for path in &rule_files {
        let rs = sonda_core::rules::load_ruleset(path)?;
        rulesets.push(rs);
    }

    if rulesets.is_empty() {
        return Err(sonda_core::error::SondaError::RulesetInvalid(
            "no rulesets specified".into(),
        ));
    }

    // Read PDF
    let pdf_bytes = std::fs::read(&pdf_file)?;

    // Extract and classify
    let extractor = PdftotextExtractor::new();
    let result = sonda_core::classify_pdf(&pdf_bytes, &extractor, &rulesets)?;

    // Output
    match output_format {
        "json" => output::json::print(&result)?,
        _ => output::table::print(&result, show_all, verbose),
    }

    Ok(())
}
