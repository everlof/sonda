use sonda_core::extraction::pdftotext::PdftotextExtractor;
use std::path::PathBuf;

use crate::output;

pub fn run(
    input_file: PathBuf,
    output_format: &str,
    output_file: Option<PathBuf>,
) -> Result<(), sonda_core::error::SondaError> {
    let input_bytes = std::fs::read(&input_file)?;

    let is_xlsx = input_file
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("xlsx"))
        .unwrap_or(false);

    let parsed = if is_xlsx {
        sonda_core::parse_sweco_xlsx(&input_bytes)?
    } else {
        let extractor = PdftotextExtractor::new();
        sonda_core::parse_pdf(&input_bytes, &extractor)?
    };

    let output_str = match output_format {
        // Use the same JSON shape that `sonda classify` consumes.
        "json" => serde_json::to_string_pretty(&parsed.reports)?,
        _ => output::table::format_parsed(&parsed),
    };

    match output_file {
        Some(path) => {
            // Always write JSON when saving to file.
            // The file format is a top-level array of AnalysisReport.
            let json = serde_json::to_string_pretty(&parsed.reports)?;
            std::fs::write(&path, json)?;
            eprintln!(
                "Parsed {} sample(s), written to {}",
                parsed.reports.len(),
                path.display()
            );
            if !parsed.warnings.is_empty() {
                for w in &parsed.warnings {
                    eprintln!("  warning: {}", w.reason);
                }
            }
            if !parsed.skipped_lines.is_empty() {
                eprintln!(
                    "  {} line(s) skipped during parsing",
                    parsed.skipped_lines.len()
                );
            }
        }
        None => {
            println!("{output_str}");
        }
    }

    Ok(())
}
