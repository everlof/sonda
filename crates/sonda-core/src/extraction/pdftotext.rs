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
    let mut out = Vec::new();
    let mut cursor = 0usize;
    let mut page_seq = 0usize;
    while let Some(page_pos_rel) = xml[cursor..].find("<page ") {
        page_seq += 1;
        let page_pos = cursor + page_pos_rel;
        let page_tag_end = match xml[page_pos..].find('>') {
            Some(v) => page_pos + v,
            None => break,
        };
        let page_tag = &xml[page_pos..=page_tag_end];
        let current_page = parse_attr_usize(page_tag, "number").unwrap_or(page_seq);

        let page_close = match xml[page_tag_end + 1..].find("</page>") {
            Some(v) => page_tag_end + 1 + v,
            None => break,
        };
        let page_body = &xml[page_tag_end + 1..page_close];

        let mut line_cursor = 0usize;
        while let Some(line_pos_rel) = page_body[line_cursor..].find("<line ") {
            let line_pos = line_cursor + line_pos_rel;
            let line_tag_end = match page_body[line_pos..].find('>') {
                Some(v) => line_pos + v,
                None => break,
            };
            let line_open_tag = &page_body[line_pos..=line_tag_end];

            let line_close_rel = match page_body[line_tag_end + 1..].find("</line>") {
                Some(v) => v,
                None => break,
            };
            let line_close = line_tag_end + 1 + line_close_rel;
            let line_body = &page_body[line_tag_end + 1..line_close];

            let bbox = parse_bbox(line_open_tag);
            let mut words = Vec::new();
            let mut word_cursor = 0usize;
            while let Some(word_pos_rel) = line_body[word_cursor..].find("<word ") {
                let word_pos = word_cursor + word_pos_rel;
                let word_tag_end = match line_body[word_pos..].find('>') {
                    Some(v) => word_pos + v,
                    None => break,
                };
                let word_close_rel = match line_body[word_tag_end + 1..].find("</word>") {
                    Some(v) => v,
                    None => break,
                };
                let word_close = word_tag_end + 1 + word_close_rel;
                let word_text = &line_body[word_tag_end + 1..word_close];
                let word = decode_xml_entities(word_text).trim().to_string();
                if !word.is_empty() {
                    words.push(word);
                }
                word_cursor = word_close + "</word>".len();
            }

            if let Some(bbox) = bbox {
                let text = words.join(" ");
                if !text.is_empty() {
                    out.push(BBoxLine {
                        page_number: current_page,
                        text,
                        bbox,
                    });
                }
            }

            line_cursor = line_close + "</line>".len();
        }

        cursor = page_close + "</page>".len();
    }

    out
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

fn decode_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
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
