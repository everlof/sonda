use sonda_core::extraction::pdftotext::PdftotextExtractor;
use std::path::PathBuf;

use crate::output;

pub fn run(
    pdf_file: PathBuf,
    output_format: &str,
    output_file: Option<PathBuf>,
) -> Result<(), sonda_core::error::SondaError> {
    let pdf_bytes = std::fs::read(&pdf_file)?;
    let extractor = PdftotextExtractor::new();
    let parsed = sonda_core::parse_pdf(&pdf_bytes, &extractor)?;

    let output_str = match output_format {
        "json" => serde_json::to_string_pretty(&parsed)?,
        _ => output::table::format_parsed(&parsed),
    };

    match output_file {
        Some(path) => {
            // Always write JSON when saving to file
            let json = serde_json::to_string_pretty(&parsed)?;
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
