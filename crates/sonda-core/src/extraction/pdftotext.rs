use crate::error::SondaError;
use crate::extraction::{BBox, LineSpan, PageContent, PdfExtractor};
use std::io::Write;
use std::process::Command;

/// PDF extraction backend using pdftotext (from poppler-utils).
///
/// Uses `pdftotext -layout` to preserve whitespace alignment of tables.
pub struct PdftotextExtractor;

impl PdftotextExtractor {
    pub fn new() -> Self {
        PdftotextExtractor
    }

    /// Check if pdftotext is available on the system.
    pub fn is_available() -> bool {
        Command::new("pdftotext")
            .arg("-v")
            .output()
            .map(|o| o.status.success() || !o.stderr.is_empty())
            .unwrap_or(false)
    }
}

impl Default for PdftotextExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl PdfExtractor for PdftotextExtractor {
    fn extract_pages(&self, pdf_bytes: &[u8]) -> Result<Vec<PageContent>, SondaError> {
        // Write PDF bytes to a temp file
        let mut tmpfile =
            tempfile::NamedTempFile::new().map_err(|e| SondaError::Extraction(e.to_string()))?;
        tmpfile
            .write_all(pdf_bytes)
            .map_err(|e| SondaError::Extraction(e.to_string()))?;
        let tmp_path = tmpfile.path().to_path_buf();

        // Run pdftotext -layout for table-friendly text extraction.
        let output = Command::new("pdftotext")
            .arg("-layout")
            .arg(&tmp_path)
            .arg("-") // output to stdout
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    SondaError::PdftotextNotFound
                } else {
                    SondaError::Extraction(format!("pdftotext failed: {}", e))
                }
            })?;

        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(SondaError::PdftotextFailed { code, stderr });
        }

        let text = String::from_utf8_lossy(&output.stdout);

        // Also extract line-level bounding boxes for highlighting.
        let bbox_lines = extract_bbox_lines(&tmp_path)?;

        // Split into pages (pdftotext uses form feed \x0c as page separator)
        let pages: Vec<PageContent> = text
            .split('\x0c')
            .enumerate()
            .map(|(i, page_text)| {
                let lines: Vec<String> = page_text.lines().map(|l| l.to_string()).collect();
                let line_spans = bbox_lines
                    .iter()
                    .filter(|b| b.page_number == i + 1)
                    .enumerate()
                    .map(|(line_index, b)| LineSpan {
                        page_number: i + 1,
                        line_index,
                        text: b.text.clone(),
                        bbox: b.bbox.clone(),
                    })
                    .collect();
                PageContent {
                    page_number: i + 1,
                    lines,
                    line_spans,
                }
            })
            .filter(|p| !p.lines.is_empty() || p.page_number == 1)
            .collect();

        Ok(pages)
    }

    fn backend_name(&self) -> &str {
        "pdftotext"
    }
}

#[derive(Debug, Clone)]
struct BBoxLine {
    page_number: usize,
    text: String,
    bbox: BBox,
}

fn extract_bbox_lines(pdf_path: &std::path::Path) -> Result<Vec<BBoxLine>, SondaError> {
    let output = Command::new("pdftotext")
        .arg("-bbox-layout")
        .arg(pdf_path)
        .arg("-")
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                SondaError::PdftotextNotFound
            } else {
                SondaError::Extraction(format!("pdftotext -bbox-layout failed: {}", e))
            }
        })?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(SondaError::PdftotextFailed { code, stderr });
    }

    let xml = String::from_utf8_lossy(&output.stdout);
    Ok(parse_bbox_xml(&xml))
}

fn parse_bbox_xml(xml: &str) -> Vec<BBoxLine> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut out: Vec<BBoxLine> = Vec::new();
    let mut page_seq: usize = 0;
    let mut current_page: usize = 0;
    let mut in_line = false;
    let mut line_bbox: Option<BBox> = None;
    let mut current_words: Vec<String> = Vec::new();
    let mut in_word = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                b"page" => {
                    page_seq += 1;
                    current_page = attr_usize(e, b"number").unwrap_or(page_seq);
                }
                b"line" => {
                    in_line = true;
                    line_bbox = attr_bbox(e);
                    current_words.clear();
                }
                b"word" if in_line => {
                    in_word = true;
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) if in_word => {
                if let Ok(text) = e.unescape() {
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        current_words.push(trimmed);
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                b"word" => {
                    in_word = false;
                }
                b"line" => {
                    if let Some(bbox) = line_bbox.take() {
                        let text = current_words.join(" ");
                        if !text.is_empty() {
                            out.push(BBoxLine {
                                page_number: current_page,
                                text,
                                bbox,
                            });
                        }
                    }
                    current_words.clear();
                    in_line = false;
                    in_word = false;
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    out
}

fn attr_f32(e: &quick_xml::events::BytesStart<'_>, name: &[u8]) -> Option<f32> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.local_name().as_ref() == name)
        .and_then(|a| std::str::from_utf8(&a.value).ok()?.parse().ok())
}

fn attr_usize(e: &quick_xml::events::BytesStart<'_>, name: &[u8]) -> Option<usize> {
    e.attributes()
        .filter_map(|a| a.ok())
        .find(|a| a.key.local_name().as_ref() == name)
        .and_then(|a| std::str::from_utf8(&a.value).ok()?.parse().ok())
}

fn attr_bbox(e: &quick_xml::events::BytesStart<'_>) -> Option<BBox> {
    Some(BBox {
        x_min: attr_f32(e, b"xMin")?,
        y_min: attr_f32(e, b"yMin")?,
        x_max: attr_f32(e, b"xMax")?,
        y_max: attr_f32(e, b"yMax")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bbox_xml_lines() {
        let xml = r#"
<doc>
  <page number="1">
    <line xMin="10.0" yMin="20.0" xMax="60.0" yMax="30.0">
      <word xMin="10.0" yMin="20.0" xMax="30.0" yMax="30.0">Arsenik</word>
      <word xMin="32.0" yMin="20.0" xMax="40.0" yMax="30.0">(As)</word>
    </line>
  </page>
</doc>
"#;
        let lines = parse_bbox_xml(xml);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].page_number, 1);
        assert_eq!(lines[0].text, "Arsenik (As)");
        assert_eq!(lines[0].bbox.x_min, 10.0);
    }

    #[test]
    fn test_bbox_lines_mapped_to_page_spans() {
        let bbox_lines = [BBoxLine {
            page_number: 1,
            text: "Arsenik (As) 68 mg/kg TS".to_string(),
            bbox: BBox {
                x_min: 10.0,
                y_min: 20.0,
                x_max: 120.0,
                y_max: 30.0,
            },
        }];

        let spans: Vec<LineSpan> = bbox_lines
            .iter()
            .filter(|b| b.page_number == 1)
            .enumerate()
            .map(|(line_index, b)| LineSpan {
                page_number: 1,
                line_index,
                text: b.text.clone(),
                bbox: b.bbox.clone(),
            })
            .collect();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].line_index, 0);
        assert_eq!(spans[0].page_number, 1);
    }
}
