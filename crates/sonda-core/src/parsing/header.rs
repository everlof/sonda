use crate::model::{Matrix, ReportHeader};

/// Extract report header information from text lines.
pub fn parse_header(lines: &[&str]) -> ReportHeader {
    let mut header = ReportHeader::default();

    for line in lines {
        let line = line.trim();

        // Detect laboratory
        if header.lab.is_none() {
            let lower = line.to_lowercase();
            if lower.contains("eurofins") {
                header.lab = Some("Eurofins".to_string());
            }
        }

        // Try to extract lab report ID (Rapport-/LaboratorieID patterns)
        if header.lab_report_id.is_none() {
            if let Some(id) = extract_after_label(line, "rapport") {
                header.lab_report_id = Some(id);
            } else if let Some(id) = extract_after_label(line, "laboratorienummer") {
                header.lab_report_id = Some(id);
            }
        }

        // Try to extract sample ID
        // Prefer Provmärkning (human label like "Väg 115 P1") over Provnummer (lab number)
        if let Some(id) = extract_after_label(line, "provmärkning") {
            header.sample_id = Some(id);
        } else if header.sample_id.is_none() {
            if let Some(id) = extract_after_label(line, "provnummer") {
                header.sample_id = Some(id);
            }
        }

        // Matrix detection
        if header.matrix.is_none() {
            if let Some(val) = extract_after_label(line, "matris") {
                header.matrix = Matrix::from_str_loose(&val);
            } else if let Some(val) = extract_after_label(line, "provtyp") {
                header.matrix = Matrix::from_str_loose(&val);
            }
            // Also check for matrix keywords anywhere in header lines
            if header.matrix.is_none() {
                let lower = line.to_lowercase();
                if lower.contains("matris") && lower.contains("jord") {
                    header.matrix = Some(Matrix::Jord);
                } else if lower.contains("matris") && lower.contains("asfalt") {
                    header.matrix = Some(Matrix::Asfalt);
                }
            }
        }

        // Project
        if header.project.is_none() {
            if let Some(val) = extract_after_label(line, "projekt") {
                header.project = Some(val);
            } else if let Some(val) = extract_after_label(line, "uppdrag") {
                header.project = Some(val);
            }
        }
    }

    header
}

/// Extract a value appearing after a label (case-insensitive).
/// Handles patterns like "Label: value" or "Label    value" (tab/space separated).
/// Truncates at the next large whitespace gap (3+ spaces) to avoid capturing
/// trailing fields from pdftotext -layout output.
fn extract_after_label(line: &str, label: &str) -> Option<String> {
    let lower = line.to_lowercase();
    if let Some(idx) = lower.find(label) {
        let after = &line[idx + label.len()..];
        // Skip colon and whitespace
        let trimmed = after.trim_start_matches(|c: char| c == ':' || c.is_whitespace());
        if trimmed.is_empty() {
            return None;
        }
        // Truncate at next large whitespace gap (3+ spaces)
        let value = if let Some(gap_pos) = trimmed.find("   ") {
            trimmed[..gap_pos].trim()
        } else {
            trimmed.trim()
        };
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header_basic() {
        let lines = [
            "Rapport: ABC123",
            "Provnummer: P001",
            "Matris: Jord",
            "Projekt: Test",
        ];
        let h = parse_header(&lines);
        assert_eq!(h.lab_report_id.as_deref(), Some("ABC123"));
        assert_eq!(h.sample_id.as_deref(), Some("P001"));
        assert_eq!(h.matrix, Some(Matrix::Jord));
        assert_eq!(h.project.as_deref(), Some("Test"));
    }

    #[test]
    fn test_matrix_asfalt() {
        let lines = ["Matris: Asfalt"];
        let h = parse_header(&lines);
        assert_eq!(h.matrix, Some(Matrix::Asfalt));
    }
}
