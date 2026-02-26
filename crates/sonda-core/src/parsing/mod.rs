pub mod header;
pub mod normalize;
pub mod values;

use crate::error::SondaError;
use crate::extraction::PageContent;
use crate::model::{AnalysisReport, AnalysisRow, Unit};
use header::parse_header;
use normalize::normalize_substance;
use values::parse_value;

#[derive(Debug, Clone)]
pub struct ParseWarning {
    pub section_index: usize,
    pub sample_id: Option<String>,
    pub reason: String,
}

/// A line that looked like substance data but could not be fully parsed.
#[derive(Debug, Clone)]
pub struct SkippedLine {
    pub line_text: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ParsedReports {
    pub reports: Vec<AnalysisReport>,
    pub warnings: Vec<ParseWarning>,
    pub skipped_lines: Vec<SkippedLine>,
}

/// Parse extracted page content into one or more AnalysisReports.
///
/// Multi-sample PDFs (where multiple "Analysrapport" sections appear)
/// are split into separate reports, each classified independently.
pub fn parse_reports(pages: &[PageContent]) -> Result<ParsedReports, SondaError> {
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
    let mut warnings = Vec::new();
    let mut skipped_lines: Vec<SkippedLine> = Vec::new();
    for (idx, section) in sections.iter().enumerate() {
        match parse_section(section) {
            Ok((report, section_skipped)) => {
                skipped_lines.extend(section_skipped);
                reports.push(report);
            }
            Err(err) => {
                // Keep parsing remaining sections and surface explicit warnings.
                let header_lines: Vec<&str> = section.iter().take(30).copied().collect();
                let header = parse_header(&header_lines);
                let sample_id = header.sample_id.or(header.lab_report_id);
                let reason = match err {
                    SondaError::ParseError(msg) => msg,
                    other => other.to_string(),
                };
                warnings.push(ParseWarning {
                    section_index: idx + 1,
                    sample_id,
                    reason,
                });
                continue;
            }
        }
    }

    if reports.is_empty() {
        return Err(SondaError::ParseError(
            "no analysis values found in report".into(),
        ));
    }

    Ok(ParsedReports {
        reports,
        warnings,
        skipped_lines,
    })
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
fn parse_section(lines: &[&str]) -> Result<(AnalysisReport, Vec<SkippedLine>), SondaError> {
    // Parse header from the first ~30 lines of this section
    let header_lines: Vec<&str> = lines.iter().take(30).copied().collect();
    let header = parse_header(&header_lines);

    // Find and parse table rows
    let (rows, skipped) = parse_table_rows(lines)?;

    if rows.is_empty() {
        return Err(SondaError::ParseError(
            "no analysis values found in section".into(),
        ));
    }

    Ok((AnalysisReport { header, rows }, skipped))
}

/// Parse table rows from text lines.
fn parse_table_rows(lines: &[&str]) -> Result<(Vec<AnalysisRow>, Vec<SkippedLine>), SondaError> {
    let mut rows = Vec::new();
    let mut skipped = Vec::new();

    for line in lines {
        let line = line.trim();
        if line.is_empty() || line.len() < 3 {
            continue;
        }

        match try_parse_row(line) {
            Ok(Some(row)) => rows.push(row),
            Ok(None) => {}
            Err(skip) => skipped.push(skip),
        }
    }

    Ok((rows, skipped))
}

/// Try to parse a single line as a substance row.
///
/// Returns `Ok(None)` for lines that clearly aren't data rows (too short,
/// header words, non-alphabetic start). Returns `Err(SkippedLine)` for lines
/// that look like substance data but have no parseable value.
fn try_parse_row(line: &str) -> Result<Option<AnalysisRow>, SkippedLine> {
    // Split the line into segments by large whitespace gaps (2+ spaces)
    let segments: Vec<&str> = split_by_whitespace_gaps(line);

    if segments.len() < 2 {
        return Ok(None);
    }

    // The first segment should be the substance name (must start with a letter)
    let name = segments[0].trim();
    if name.is_empty()
        || !name
            .chars()
            .next()
            .map(|c| c.is_alphabetic())
            .unwrap_or(false)
    {
        return Ok(None);
    }

    // Skip header-like lines
    let name_lower = name.to_lowercase();
    if is_header_word(&name_lower) {
        return Ok(None);
    }

    // Look for a value in subsequent segments
    let mut last_err: Option<String> = None;
    for segment in &segments[1..] {
        let segment = segment.trim();
        match parse_value(segment) {
            Ok(Some(value)) => {
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

                return Ok(Some(AnalysisRow {
                    raw_name: name.to_string(),
                    normalized_name: normalized,
                    value,
                    unit,
                }));
            }
            Ok(None) => {}
            Err(e) => {
                last_err = Some(format!("{}", e));
            }
        }
    }

    let reason = last_err.unwrap_or_else(|| "no parseable numeric value found".to_string());
    Err(SkippedLine {
        line_text: line.to_string(),
        reason,
    })
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
        let row = try_parse_row("Arsenik (As)     68     mg/kg TS")
            .unwrap()
            .unwrap();
        assert_eq!(row.normalized_name, "arsenik");
        assert_eq!(row.value, AnalysisValue::Measured(dec!(68)));
    }

    #[test]
    fn test_try_parse_below_detection_row() {
        let row = try_parse_row("Kvicksilver (Hg)     < 0.030     mg/kg TS")
            .unwrap()
            .unwrap();
        assert_eq!(row.normalized_name, "kvicksilver");
        assert_eq!(row.value, AnalysisValue::BelowDetection(dec!(0.030)));
    }

    #[test]
    fn test_header_line_skipped() {
        assert!(try_parse_row("Analys     Resultat     Enhet")
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_unparseable_value_returns_skipped() {
        let result = try_parse_row("Bly (Pb)     n.d.     mg/kg TS");
        assert!(result.is_err());
        let skip = result.unwrap_err();
        assert!(skip.reason.contains("invalid number"));
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
