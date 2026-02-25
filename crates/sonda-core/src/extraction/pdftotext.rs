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
                let line_spans = match_layout_lines_to_bbox(i + 1, &lines, &bbox_lines);
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
    let mut out = Vec::new();
    let mut current_page: Option<usize> = None;
    let mut current_bbox: Option<BBox> = None;
    let mut current_words: Vec<String> = Vec::new();

    for raw in xml.lines() {
        let line = raw.trim();

        if line.starts_with("<page ") {
            current_page = parse_attr_usize(line, "number");
            continue;
        }

        if line.starts_with("<line ") {
            current_bbox = parse_bbox(line);
            current_words.clear();
            continue;
        }

        if line.starts_with("<word ") {
            if let Some(word_text) = parse_word_text(line) {
                let w = decode_xml_entities(&word_text).trim().to_string();
                if !w.is_empty() {
                    current_words.push(w);
                }
            }
            continue;
        }

        if line.starts_with("</line>") {
            if let (Some(page_number), Some(bbox)) = (current_page, current_bbox.take()) {
                let text = current_words.join(" ");
                if !text.is_empty() {
                    out.push(BBoxLine {
                        page_number,
                        text,
                        bbox,
                    });
                }
            }
            current_words.clear();
        }
    }

    out
}

fn match_layout_lines_to_bbox(
    page_number: usize,
    lines: &[String],
    bbox_lines: &[BBoxLine],
) -> Vec<LineSpan> {
    let mut spans = Vec::new();
    let mut used = vec![false; bbox_lines.len()];

    for (line_index, line) in lines.iter().enumerate() {
        let norm = normalize_ws(line);
        if norm.is_empty() {
            continue;
        }

        if let Some((i, b)) = bbox_lines.iter().enumerate().find(|(i, b)| {
            !used[*i] && b.page_number == page_number && normalize_ws(&b.text) == norm
        }) {
            used[i] = true;
            spans.push(LineSpan {
                page_number,
                line_index,
                text: line.clone(),
                bbox: b.bbox.clone(),
            });
        }
    }

    spans
}

fn parse_attr_usize(tag: &str, name: &str) -> Option<usize> {
    parse_attr(tag, name)?.parse().ok()
}

fn parse_attr_f32(tag: &str, name: &str) -> Option<f32> {
    parse_attr(tag, name)?.parse().ok()
}

fn parse_attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("{}=\"", name);
    let start = tag.find(&needle)? + needle.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

fn parse_bbox(line_tag: &str) -> Option<BBox> {
    Some(BBox {
        x_min: parse_attr_f32(line_tag, "xMin")?,
        y_min: parse_attr_f32(line_tag, "yMin")?,
        x_max: parse_attr_f32(line_tag, "xMax")?,
        y_max: parse_attr_f32(line_tag, "yMax")?,
    })
}

fn parse_word_text(word_tag: &str) -> Option<String> {
    let start = word_tag.find('>')? + 1;
    let end = word_tag.rfind("</word>")?;
    Some(word_tag[start..end].to_string())
}

fn decode_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn normalize_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
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
    fn test_match_layout_lines_to_bbox() {
        let bbox_lines = vec![BBoxLine {
            page_number: 1,
            text: "Arsenik (As) 68 mg/kg TS".to_string(),
            bbox: BBox {
                x_min: 10.0,
                y_min: 20.0,
                x_max: 120.0,
                y_max: 30.0,
            },
        }];

        let lines = vec!["Arsenik (As)   68   mg/kg TS".to_string()];
        let spans = match_layout_lines_to_bbox(1, &lines, &bbox_lines);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].line_index, 0);
        assert_eq!(spans[0].page_number, 1);
    }
}
