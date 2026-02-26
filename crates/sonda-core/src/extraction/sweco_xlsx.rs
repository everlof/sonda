use std::io::Cursor;

use calamine::{Reader, Xlsx};
use rust_decimal::Decimal;

use crate::error::SondaError;
use crate::model::{AnalysisReport, AnalysisRow, AnalysisValue, Matrix, ReportHeader, Unit};
use crate::parsing::normalize::normalize_substance;
use crate::parsing::ParsedReports;

/// Parse a Sweco "AVFALLSKLASSNING@SWECO" xlsx file into structured reports.
///
/// Returns the same `ParsedReports` type that `parse_pdf()` produces, so the
/// result slots directly into `classify_reports()`.
pub fn parse_sweco_xlsx(bytes: &[u8]) -> Result<ParsedReports, SondaError> {
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> = calamine::open_workbook_from_rs(cursor)
        .map_err(|e| SondaError::ParseError(format!("failed to open xlsx: {e}")))?;

    let sheet = workbook
        .worksheet_range("Sammanställning")
        .map_err(|e| SondaError::ParseError(format!("sheet 'Sammanställning' not found: {e}")))?;

    // Verify format marker in row 1 (0-indexed row 0)
    let marker = sheet.get_value((0, 0)).and_then(cell_as_string);
    match marker {
        Some(ref s) if s.contains("AVFALLSKLASSNING@SWECO") => {}
        _ => {
            return Err(SondaError::ParseError(
                "not a Sweco AVFALLSKLASSNING file (missing marker in row 1)".into(),
            ));
        }
    }

    // Row 3 (0-indexed row 2): col A = sample name, col G = date
    let sample_id = sheet.get_value((2, 0)).and_then(cell_as_string);
    let date = sheet.get_value((2, 6)).and_then(cell_as_string);

    // Parse substance rows starting at row 17 (0-indexed row 16) until first empty row
    let mut rows = Vec::new();
    let mut skipped_lines = Vec::new();
    let mut row_idx: u32 = 16;

    loop {
        let name_cell = sheet.get_value((row_idx, 0));
        let raw_name = match name_cell.and_then(cell_as_string) {
            Some(n) if !n.is_empty() => n,
            _ => break, // Empty row = end of data
        };

        let value_cell = sheet.get_value((row_idx, 1));
        match cell_as_f64(value_cell) {
            Some(f) => {
                let decimal = f64_to_decimal(f);
                let normalized = normalize_substance(&raw_name);
                rows.push(AnalysisRow {
                    raw_name,
                    normalized_name: normalized,
                    value: AnalysisValue::Measured(decimal),
                    unit: Unit::MgPerKgTs,
                });
            }
            None => {
                let cell_text = value_cell.map(|c| format!("{c}")).unwrap_or_default();
                if !cell_text.is_empty() {
                    skipped_lines.push(crate::parsing::SkippedLine {
                        line_text: format!("{raw_name}: {cell_text}"),
                        reason: "non-numeric value in xlsx".into(),
                    });
                }
            }
        }

        row_idx += 1;
    }

    if rows.is_empty() {
        return Err(SondaError::ParseError(
            "no substance data found in xlsx".into(),
        ));
    }

    let header = ReportHeader {
        lab: Some("Sweco".into()),
        sample_id,
        matrix: Some(Matrix::Jord),
        date,
        ..Default::default()
    };

    Ok(ParsedReports {
        reports: vec![AnalysisReport { header, rows }],
        warnings: vec![],
        skipped_lines,
    })
}

fn cell_as_string(cell: &calamine::Data) -> Option<String> {
    match cell {
        calamine::Data::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        calamine::Data::Float(f) => Some(f.to_string()),
        calamine::Data::Int(i) => Some(i.to_string()),
        calamine::Data::DateTime(dt) => Some(dt.to_string()),
        calamine::Data::Empty => None,
        _ => Some(format!("{cell}")),
    }
}

fn cell_as_f64(cell: Option<&calamine::Data>) -> Option<f64> {
    match cell? {
        calamine::Data::Float(f) => Some(*f),
        calamine::Data::Int(i) => Some(*i as f64),
        _ => None,
    }
}

/// Convert f64 to Decimal, preserving reasonable precision.
///
/// Uses string round-trip to avoid floating-point artifacts
/// (e.g., 0.0035_f64 becoming 0.00349999...).
fn f64_to_decimal(f: f64) -> Decimal {
    // Format with enough precision to capture the original value,
    // then parse as Decimal.
    let s = format!("{f}");
    s.parse::<Decimal>().unwrap_or_else(|_| {
        // Fallback: use from_f64_retain
        Decimal::try_from(f).unwrap_or_default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn f64_to_decimal_preserves_precision() {
        assert_eq!(f64_to_decimal(0.0035), dec!(0.0035));
        assert_eq!(f64_to_decimal(68.0), dec!(68));
        assert_eq!(f64_to_decimal(1.23), dec!(1.23));
    }
}
