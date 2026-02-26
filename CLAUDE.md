# CLAUDE.md

## Build & Test

```bash
cargo build                    # build workspace
cargo test --workspace         # run all tests (83 unit + 9 integration)
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
- `docs/pipeline.md` — **pipeline architecture diagram (Mermaid)** — keep up to date when changing input formats, processing steps, engines, or output
- `.github/workflows/ci.yml` — CI (fmt, clippy, check, test)

### Key Source Files

- `sonda-core/src/lib.rs` — `classify_pdf()`, `parse_sweco_xlsx()`, `classify_reports()` API entry points
- `sonda-core/src/model.rs` — core data types (AnalysisValue, Matrix, AnalysisRow, AnalysisReport)
- `sonda-core/src/classify/engine.rs` — threshold classification logic with reason generation
- `sonda-core/src/classify/hp_engine.rs` — HP criteria evaluation engine (FA classification)
- `sonda-core/src/classify/outcome.rs` — result types including HpDetails, HpCriterionDetail
- `sonda-core/src/clp/database.rs` — CLP substance database (embedded JSON, LazyLock)
- `sonda-core/src/clp/schema.rs` — serde types for CLP data (ClpSubstance, SpeciationTable)
- `sonda-core/src/clp/speciation.rs` — substance resolution (lab name → CLP compound)
- `sonda-core/src/parsing/normalize.rs` — substance name normalization + alias map
- `sonda-core/src/parsing/values.rs` — value parsing ("68", "< 0.030")
- `sonda-core/src/extraction/pdftotext.rs` — pdftotext subprocess backend + quick-xml bbox parser
- `sonda-core/src/extraction/sweco_xlsx.rs` — Sweco AVFALLSKLASSNING xlsx parser (calamine)
- `sonda-core/src/rules/schema.rs` — serde types for rule JSON format
- `sonda-core/src/rules/builtin.rs` — embedded predefined rulesets (nv, asfalt, fa)

## Conventions

- **Always update `docs/pipeline.md`** when adding/changing input formats, processing steps, classification engines, or output formats
- All numeric values use `rust_decimal::Decimal`, never f64
- Substance names are normalized to lowercase snake_case canonical keys (see alias map in `normalize.rs`)
- Rule JSON threshold values are strings, not numbers, to preserve decimal precision
- Classification reasons are generated as human-readable strings on every `SubstanceResult`
- PDF extraction is behind a trait (`PdfExtractor`) for pluggability; integration tests use a `MockExtractor`
- Bbox XML from `pdftotext -bbox-layout` is parsed with `quick-xml` (event-based reader)
- Lines with valid substance names but unparseable values produce `SkippedLine` diagnostics (surfaced as trace warnings)
- Tests for parsing and classification use `rust_decimal_macros::dec!()` macro
- CLP substance data embedded from `rules/clp-substances.json` via `include_str!`
- Speciation: metals use worst-case compound assumption with molecular weight conversion factors
- HP evaluation uses % w/w (divide mg/kg by 10000)
- HP engine is separate from threshold engine — both produce `RuleSetResult`
- `HpDetails` on `RuleSetResult` is `Option` (None for threshold rulesets, Some for HP)

## External Dependencies

- Requires `pdftotext` (from poppler-utils) installed on the system for PDF extraction
- Sample Eurofins PDFs for integration testing are in `/Users/david/Downloads/asdbsaodh/`
