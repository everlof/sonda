use sonda_core::extraction::pdftotext::PdftotextExtractor;
use sonda_core::model::AnalysisReport;
use sonda_core::rules::builtin;
use sonda_core::rules::schema::RuleSetDef;
use sonda_core::ClassifyOptions;
use std::path::PathBuf;

use crate::output;

fn parse_reports_json(
    json_bytes: &[u8],
) -> Result<Vec<AnalysisReport>, sonda_core::error::SondaError> {
    let reports = serde_json::from_slice::<Vec<AnalysisReport>>(json_bytes)?;
    Ok(reports)
}

fn looks_like_json(bytes: &[u8]) -> bool {
    bytes
        .iter()
        .copied()
        .find(|b| !b.is_ascii_whitespace())
        .map(|b| b == b'[' || b == b'{')
        .unwrap_or(false)
}

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

    // Determine input type by extension; also allow extension-less JSON files.
    let ext_is_json = input_file
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("json"))
        .unwrap_or(false);
    let input_bytes = std::fs::read(&input_file)?;
    let should_parse_json = ext_is_json || looks_like_json(&input_bytes);

    let result = if should_parse_json {
        // Load pre-parsed reports from JSON.
        // Expected shape: top-level array of AnalysisReport.
        let reports = parse_reports_json(&input_bytes)?;
        sonda_core::classify_reports(&reports, &rulesets, &options)?
    } else {
        // Parse and classify PDF.
        let extractor = PdftotextExtractor::new();
        sonda_core::classify_pdf(&input_bytes, &extractor, &rulesets, &options)?
    };

    // Output
    match output_format {
        "json" => output::json::print(&result)?,
        _ => output::table::print(&result, show_all, verbose),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_reports_json;

    #[test]
    fn parse_reports_json_accepts_array_shape() {
        let json = br#"
[
  {
    "header": {
      "lab": "Eurofins",
      "sample_id": "P001",
      "matrix": "jord"
    },
    "rows": [
      {
        "raw_name": "Arsenik (As)",
        "normalized_name": "arsenik",
        "value": { "Measured": "15" },
        "unit": "mg/kg TS"
      }
    ]
  }
]
"#;
        let reports = parse_reports_json(json).expect("array JSON should parse");
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].header.sample_id.as_deref(), Some("P001"));
    }
}
