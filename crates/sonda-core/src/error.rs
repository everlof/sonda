use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum SondaError {
    #[error("PDF extraction failed: {0}")]
    Extraction(String),

    #[error("pdftotext not found. Install poppler: brew install poppler (macOS) or apt install poppler-utils (Linux)")]
    PdftotextNotFound,

    #[error("pdftotext failed with exit code {code}: {stderr}")]
    PdftotextFailed { code: i32, stderr: String },

    #[error("failed to parse report: {0}")]
    ParseError(String),

    #[error("failed to load ruleset from {path}: {reason}")]
    RulesetLoad { path: PathBuf, reason: String },

    #[error("invalid ruleset: {0}")]
    RulesetInvalid(String),

    #[error("unsupported report format: {0}. Currently only Eurofins reports are supported.")]
    UnsupportedReport(String),

    #[error("report matrix '{matrix}' does not match any of the provided rulesets")]
    MatrixMismatch { matrix: String },

    #[error("no substances matched between report and ruleset")]
    NoMatches,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
