pub mod pdftotext;
pub mod table;

use crate::error::SondaError;

/// Content extracted from a single page of a PDF.
#[derive(Debug, Clone)]
pub struct PageContent {
    pub page_number: usize,
    pub lines: Vec<String>,
}

/// Trait for PDF text extraction backends.
pub trait PdfExtractor: Send + Sync {
    /// Extract text content from PDF bytes, returning one PageContent per page.
    fn extract_pages(&self, pdf_bytes: &[u8]) -> Result<Vec<PageContent>, SondaError>;

    /// Name of this extraction backend (for diagnostics).
    fn backend_name(&self) -> &str;
}
