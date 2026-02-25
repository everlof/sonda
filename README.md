# Sonda

Waste classification tool for contaminated soil and asphalt. Takes Eurofins PDF lab reports and classifies substances against regulatory thresholds (KM/MKM/FA/IFA).

## Quick Start

```
# Requires pdftotext (poppler)
brew install poppler        # macOS
apt install poppler-utils   # Linux

# Build
cargo build --release

# Classify a report
sonda classify report.pdf
sonda classify report.pdf --verbose
sonda classify report.pdf --output json
sonda classify report.pdf --show-all --verbose
```

## Commands

### classify

```
sonda classify <PDF_FILE> [OPTIONS]
    -r, --rules <FILE>     Custom JSON rule file(s)
    -p, --preset <NAME>    Predefined ruleset(s) (default: all presets)
    -o, --output <FORMAT>  table (default) or json
    --show-all             Show all substances, not just exceedances
    --verbose              Show detailed per-substance reasoning
```

When no `--preset` or `--rules` is given, all built-in presets are run (`nv`, `asfalt`, `fa`).

### rules

```
sonda rules list              List predefined rulesets
sonda rules explain <PRESET>  Explain a ruleset with all thresholds
sonda rules schema            Print JSON schema for custom rule files
sonda rules validate <FILE>   Validate a custom rule file
```

## Predefined Rulesets

| Preset | Name | Categories |
|--------|------|------------|
| `nv` | Naturvårdsverkets generella riktvärden (2025.1) | KM, MKM |
| `asfalt` | PAH-16 asfaltklassificering (2025.1) | Ren, Förorenad, Farligt avfall |
| `fa` | Farligt avfall (HP-bedömning) | FA, Icke FA |

## Custom Rules

Create a JSON file with your own thresholds:

```json
{
  "name": "Project X thresholds",
  "version": "1.0",
  "categories": ["Clean", "Moderate", "Contaminated"],
  "rules": [
    { "substance": "bly", "thresholds": { "Clean": "20", "Moderate": "100", "Contaminated": "500" } },
    { "substance": "arsenik", "thresholds": { "Clean": "5", "Moderate": "15", "Contaminated": "40" } }
  ]
}
```

```
sonda classify report.pdf --rules my-rules.json
```

Run `sonda rules schema` for the full schema reference.

## Architecture

Cargo workspace with two crates:

- **sonda-core** — library with extraction, parsing, classification engine. Exposes `classify_pdf()` as the main API entry point.
- **sonda-cli** — thin CLI using clap.

Key design decisions:
- `rust_decimal::Decimal` for all values (no float rounding at classification boundaries)
- Substance name normalization as the join key between report and rules
- PDF extraction via pluggable `PdfExtractor` trait (Phase 1: pdftotext subprocess)
- Rules are pure JSON data, embedded at compile time for presets
- Every classification decision carries a human-readable reason string

## BBox Viewer (Trace Highlight Debug UI)

A minimal browser UI is included to test `trace.entries[].evidence_spans` overlays.

```
# 1) Generate JSON with trace data
sonda classify report.pdf --output json > result.json

# 2) Start a static file server from repo root
python3 -m http.server 8000

# 3) Open the viewer
http://localhost:8000/tools/bbox-viewer/
```

Then load:
- the original PDF file
- the generated `result.json`

Click an entry in the left panel to highlight its matched spans on the PDF page.
