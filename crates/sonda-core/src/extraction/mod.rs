pub mod pdftotext;
pub mod sweco_xlsx;
pub mod table;

use crate::error::SondaError;

#[derive(Debug, Clone)]
pub struct BBox {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

#[derive(Debug, Clone)]
pub struct LineSpan {
    pub page_number: usize,
    pub line_index: usize,
    pub text: String,
    pub bbox: BBox,
}

/// Content extracted from a single page of a PDF.
#[derive(Debug, Clone)]
pub struct PageContent {
    pub page_number: usize,
    pub lines: Vec<String>,
    pub line_spans: Vec<LineSpan>,
}

/// Trait for PDF text extraction backends.
pub trait PdfExtractor: Send + Sync {
    /// Extract text content from PDF bytes, returning one PageContent per page.
    fn extract_pages(&self, pdf_bytes: &[u8]) -> Result<Vec<PageContent>, SondaError>;

    /// Name of this extraction backend (for diagnostics).
    fn backend_name(&self) -> &str;
}
