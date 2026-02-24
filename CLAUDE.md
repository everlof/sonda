# CLAUDE.md

## Build & Test

```bash
cargo build                    # build workspace
cargo test --workspace         # run all tests (42 unit tests)
cargo test -p sonda-core       # core library tests only
cargo build --release          # optimized build
```

The CLI binary is `sonda` (defined in sonda-cli).

## Project Structure

Cargo workspace with two crates:

- `crates/sonda-core/` — library: extraction, parsing, normalization, rules, classification engine
- `crates/sonda-cli/` — CLI binary using clap
- `rules/` — predefined ruleset JSON files (embedded via `include_str!`)
- `docs/research/` — regulatory research documentation

### Key Source Files

- `sonda-core/src/lib.rs` — `classify_pdf()` main API entry point
- `sonda-core/src/model.rs` — core data types (AnalysisValue, Matrix, AnalysisRow, AnalysisReport)
- `sonda-core/src/classify/engine.rs` — classification logic with reason generation
- `sonda-core/src/parsing/normalize.rs` — substance name normalization + alias map
- `sonda-core/src/parsing/values.rs` — value parsing ("68", "< 0.030")
- `sonda-core/src/extraction/pdftotext.rs` — pdftotext subprocess backend
- `sonda-core/src/rules/schema.rs` — serde types for rule JSON format
- `sonda-core/src/rules/builtin.rs` — embedded predefined rulesets

## Conventions

- All numeric values use `rust_decimal::Decimal`, never f64
- Substance names are normalized to lowercase snake_case canonical keys (see alias map in `normalize.rs`)
- Rule JSON threshold values are strings, not numbers, to preserve decimal precision
- Classification reasons are generated as human-readable strings on every `SubstanceResult`
- PDF extraction is behind a trait (`PdfExtractor`) for pluggability
- Tests for parsing and classification use `rust_decimal_macros::dec!()` macro

## External Dependencies

- Requires `pdftotext` (from poppler-utils) installed on the system for PDF extraction
- Sample Eurofins PDFs for integration testing are in `/Users/david/Downloads/asdbsaodh/`
