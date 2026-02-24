pub mod header;
pub mod normalize;
pub mod values;

use crate::error::SondaError;
use crate::extraction::PageContent;
use crate::model::{AnalysisReport, AnalysisRow, Unit};
use header::parse_header;
use normalize::normalize_substance;
use values::parse_value;

/// Parse extracted page content into one or more AnalysisReports.
///
/// Multi-sample PDFs (where multiple "Analysrapport" sections appear)
/// are split into separate reports, each classified independently.
pub fn parse_reports(pages: &[PageContent]) -> Result<Vec<AnalysisReport>, SondaError> {
    let all_lines: Vec<&str> = pages
        .iter()
        .flat_map(|p| p.lines.iter().map(|s| s.as_str()))
        .collect();

    if all_lines.is_empty() {
        return Err(SondaError::ParseError(
            "no text content found in PDF".into(),
        ));
    }

    // Split into sections on "Analysrapport" boundaries
    let sections = split_into_sections(&all_lines);

    let mut reports = Vec::new();
    for section in &sections {
        match parse_section(section) {
            Ok(report) => reports.push(report),
            Err(_) => {
                // Skip sections that don't parse (e.g., cover pages)
                continue;
            }
        }
    }

    if reports.is_empty() {
        return Err(SondaError::ParseError(
            "no analysis values found in report".into(),
        ));
    }

    Ok(reports)
}

/// Split lines into sections, each starting at an "Analysrapport" header.
/// If no "Analysrapport" header is found, treat the whole document as one section.
fn split_into_sections<'a>(lines: &[&'a str]) -> Vec<Vec<&'a str>> {
    let mut sections = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut found_header = false;

    for &line in lines {
        let trimmed = line.trim();
        if trimmed == "Analysrapport" {
            if found_header && !current.is_empty() {
                sections.push(current);
                current = Vec::new();
            }
            found_header = true;
        }
        current.push(line);
    }

    if !current.is_empty() {
        if found_header {
            sections.push(current);
        } else {
            // No "Analysrapport" header found — treat as single section
            sections.push(current);
        }
    }

    sections
}

/// Parse a single section (one sample) into an AnalysisReport.
fn parse_section(lines: &[&str]) -> Result<AnalysisReport, SondaError> {
    // Parse header from the first ~30 lines of this section
    let header_lines: Vec<&str> = lines.iter().take(30).copied().collect();
    let header = parse_header(&header_lines);

    // Find and parse table rows
    let rows = parse_table_rows(lines)?;

    if rows.is_empty() {
        return Err(SondaError::ParseError(
            "no analysis values found in section".into(),
        ));
    }

    Ok(AnalysisReport { header, rows })
}

/// Parse table rows from text lines.
fn parse_table_rows(lines: &[&str]) -> Result<Vec<AnalysisRow>, SondaError> {
    let mut rows = Vec::new();

    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.len() < 3 {
            continue;
        }

        if let Some(row) = try_parse_row(line) {
            rows.push(row);
        }
    }

    Ok(rows)
}

/// Try to parse a single line as a substance row.
///
/// Returns None if the line doesn't look like a data row.
fn try_parse_row(line: &str) -> Option<AnalysisRow> {
    // Split the line into segments by large whitespace gaps (2+ spaces)
    let segments: Vec<&str> = split_by_whitespace_gaps(line);

    if segments.len() < 2 {
        return None;
    }

    // The first segment should be the substance name (must start with a letter)
    let name = segments[0].trim();
    if name.is_empty() || !name.chars().next()?.is_alphabetic() {
        return None;
    }

    // Skip header-like lines
    let name_lower = name.to_lowercase();
    if is_header_word(&name_lower) {
        return None;
    }

    // Look for a value in subsequent segments
    for segment in &segments[1..] {
        let segment = segment.trim();
        if let Ok(Some(value)) = parse_value(segment) {
            let normalized = normalize_substance(name);

            // Try to detect unit from remaining segments
            let unit = segments
                .iter()
                .find_map(|s| {
                    let s = s.trim().to_lowercase();
                    if s.contains("mg/kg") {
                        Some(Unit::from_str_loose(&s))
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            return Some(AnalysisRow {
                raw_name: name.to_string(),
                normalized_name: normalized,
                value,
                unit,
            });
        }
    }

    None
}

/// Split a line by gaps of 2+ whitespace characters.
fn split_by_whitespace_gaps(line: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = None;
    let mut space_count = 0;

    for (i, c) in line.char_indices() {
        if c.is_whitespace() {
            space_count += 1;
            if space_count == 2 {
                if let Some(s) = start {
                    let end = i - 1; // exclude the first space
                    segments.push(&line[s..end]);
                    start = None;
                }
            }
        } else {
            if start.is_none() {
                start = Some(i);
            }
            space_count = 0;
        }
    }

    if let Some(s) = start {
        segments.push(&line[s..]);
    }

    segments
}

/// Check if a word is a table header rather than substance data.
fn is_header_word(s: &str) -> bool {
    matches!(
        s,
        "analys"
            | "parameter"
            | "resultat"
            | "result"
            | "enhet"
            | "unit"
            | "metod"
            | "method"
            | "mätosäkerhet"
            | "uncertainty"
            | "rapport"
            | "provnummer"
            | "matris"
            | "provmärkning"
            | "sida"
            | "page"
            | "laboratorium"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::AnalysisValue;
    use rust_decimal_macros::dec;

    #[test]
    fn test_split_by_whitespace_gaps() {
        let segments = split_by_whitespace_gaps("Arsenik (As)     68     mg/kg TS");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], "Arsenik (As)");
    }

    #[test]
    fn test_try_parse_measured_row() {
        let row = try_parse_row("Arsenik (As)     68     mg/kg TS").unwrap();
        assert_eq!(row.normalized_name, "arsenik");
        assert_eq!(row.value, AnalysisValue::Measured(dec!(68)));
    }

    #[test]
    fn test_try_parse_below_detection_row() {
        let row = try_parse_row("Kvicksilver (Hg)     < 0.030     mg/kg TS").unwrap();
        assert_eq!(row.normalized_name, "kvicksilver");
        assert_eq!(row.value, AnalysisValue::BelowDetection(dec!(0.030)));
    }

    #[test]
    fn test_header_line_skipped() {
        assert!(try_parse_row("Analys     Resultat     Enhet").is_none());
    }

    #[test]
    fn test_split_into_sections_single() {
        let lines = vec!["Header", "Analysrapport", "Data line 1", "Data line 2"];
        let sections = split_into_sections(&lines);
        assert_eq!(sections.len(), 1);
    }

    #[test]
    fn test_split_into_sections_multi() {
        let lines = vec![
            "Header",
            "Analysrapport",
            "Provnummer: 001",
            "Data 1",
            "Analysrapport",
            "Provnummer: 002",
            "Data 2",
        ];
        let sections = split_into_sections(&lines);
        assert_eq!(sections.len(), 2);
        assert!(sections[0].contains(&"Provnummer: 001"));
        assert!(sections[1].contains(&"Provnummer: 002"));
    }

    #[test]
    fn test_split_into_sections_no_header() {
        let lines = vec!["Line 1", "Line 2", "Line 3"];
        let sections = split_into_sections(&lines);
        assert_eq!(sections.len(), 1);
    }
}
