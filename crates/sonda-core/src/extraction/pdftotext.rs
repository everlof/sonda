use crate::error::SondaError;
use crate::extraction::{PageContent, PdfExtractor};
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

        // Run pdftotext -layout
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

        // Split into pages (pdftotext uses form feed \x0c as page separator)
        let pages: Vec<PageContent> = text
            .split('\x0c')
            .enumerate()
            .map(|(i, page_text)| {
                let lines: Vec<String> = page_text.lines().map(|l| l.to_string()).collect();
                PageContent {
                    page_number: i + 1,
                    lines,
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
