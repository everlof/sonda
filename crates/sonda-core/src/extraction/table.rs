use crate::extraction::PageContent;

/// Reconstruct table data from pdftotext -layout output.
///
/// pdftotext -layout preserves column alignment using spaces.
/// This module helps identify table boundaries and extract columns.
///
/// Detect if a line looks like a table header row.
pub fn is_table_header(line: &str) -> bool {
    let lower = line.to_lowercase();
    let header_keywords = [
        "analys",
        "parameter",
        "resultat",
        "enhet",
        "metod",
        "mätosäkerhet",
    ];
    let count = header_keywords
        .iter()
        .filter(|kw| lower.contains(*kw))
        .count();
    count >= 2
}

/// Find the table region(s) within page content.
/// Returns ranges of line indices that appear to be table data.
pub fn find_table_regions(pages: &[PageContent]) -> Vec<TableRegion> {
    let mut regions = Vec::new();

    for page in pages {
        let mut in_table = false;
        let mut table_start = 0;

        for (i, line) in page.lines.iter().enumerate() {
            if is_table_header(line) {
                in_table = true;
                table_start = i + 1; // data starts after header
                continue;
            }

            if in_table {
                let trimmed = line.trim();
                // End of table: empty line or page footer
                if trimmed.is_empty()
                    || trimmed.starts_with("Sida")
                    || trimmed.starts_with("Page")
                    || trimmed.contains("Eurofins")
                    || trimmed.starts_with("---")
                {
                    if i > table_start {
                        regions.push(TableRegion {
                            page_number: page.page_number,
                            start_line: table_start,
                            end_line: i,
                        });
                    }
                    in_table = false;
                }
            }
        }

        // If we reached end of page while in a table
        if in_table && page.lines.len() > table_start {
            regions.push(TableRegion {
                page_number: page.page_number,
                start_line: table_start,
                end_line: page.lines.len(),
            });
        }
    }

    regions
}

#[derive(Debug, Clone)]
pub struct TableRegion {
    pub page_number: usize,
    pub start_line: usize,
    pub end_line: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_table_header() {
        assert!(is_table_header(
            "  Analys           Resultat    Enhet     Metod"
        ));
        assert!(!is_table_header("  Arsenik (As)     68     mg/kg TS"));
    }

    #[test]
    fn test_find_table_regions() {
        let pages = vec![PageContent {
            page_number: 1,
            lines: vec![
                "Header stuff".into(),
                "  Analys           Resultat    Enhet     Metod".into(),
                "  Arsenik (As)     68          mg/kg TS  SS-EN".into(),
                "  Bly (Pb)         120         mg/kg TS  SS-EN".into(),
                "".into(),
                "Footer".into(),
            ],
            line_spans: vec![],
        }];

        let regions = find_table_regions(&pages);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].start_line, 2);
        assert_eq!(regions[0].end_line, 4);
    }
}
