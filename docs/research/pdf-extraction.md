# PDF Extraction Approach

## Requirements

Eurofins lab reports are PDFs with tabular data. The extraction must:
1. Preserve column alignment (substance name, value, unit, method)
2. Handle multi-page reports
3. Handle below-detection-limit values ("< 0.030")
4. Work reliably across different Eurofins report formats

## Evaluated Approaches

### Rust PDF Crates

| Crate | License | Approach | Verdict |
|---|---|---|---|
| `pdf-extract` | MIT | Text extraction, basic | Poor table support |
| `lopdf` | MIT | Low-level PDF object access | No text extraction |
| `mupdf` (Rust bindings) | AGPL | Full PDF rendering | License incompatible |
| `pdfium-render` | MIT | Chrome's PDFium bindings | Good, but complex setup |
| `pdf` (PDF 2.0) | MIT | PDF parsing | Incomplete text support |

### External Tools

| Tool | License | Approach | Verdict |
|---|---|---|---|
| `pdftotext` (poppler) | GPL | CLI text extraction | **Selected for Phase 1** |
| `pdfplumber` (Python) | MIT | Python table extraction | Wrong ecosystem |
| Tesseract OCR | Apache 2.0 | OCR-based | Overkill for text PDFs |

## Selected Approach: pdftotext (Phase 1)

`pdftotext -layout` from the poppler-utils package was selected because:

1. **Layout preservation**: The `-layout` flag maintains whitespace alignment, which is critical for correctly associating substance names with their values in Eurofins tables.
2. **Reliability**: poppler is the most mature open-source PDF library.
3. **Simplicity**: Subprocess call, no native dependencies to compile.
4. **Availability**: Pre-installed or easily installed on all platforms (`brew install poppler` on macOS, `apt install poppler-utils` on Linux).

### Trade-offs

- **External dependency**: Requires pdftotext to be installed on the system.
- **Performance**: Process spawning overhead (negligible for single reports).
- **No character positions**: Unlike pdfium-render, we don't get exact character coordinates. Table reconstruction relies on whitespace gap heuristics.

## Architecture: Pluggable Trait

The `PdfExtractor` trait allows swapping backends without changing the rest of the pipeline:

```rust
trait PdfExtractor: Send + Sync {
    fn extract_pages(&self, pdf_bytes: &[u8]) -> Result<Vec<PageContent>, SondaError>;
    fn backend_name(&self) -> &str;
}
```

### Future Backends (Phase 2+)

- **pdfium-render**: Native Rust bindings to PDFium. Would provide character-level positioning for more robust table reconstruction. Requires downloading the PDFium binary.
- **LLM-based extraction**: For reports with unusual layouts that resist pattern-based parsing. Would use a multimodal model to extract structured data from PDF pages rendered as images.

## Table Reconstruction

The current heuristic-based approach:

1. Identify table header lines (contain keywords like "Analys", "Resultat", "Enhet")
2. Parse data rows by splitting on large whitespace gaps (2+ spaces)
3. First segment = substance name, subsequent segments = value, unit, method
4. Value parsing handles "68", "< 0.030", "0,030" (Swedish comma)

This works well for standard Eurofins reports. Edge cases include:
- Wrapped lines (substance name or method on two lines)
- Footnote markers (*, a), b))
- Mixed table formats within the same report
