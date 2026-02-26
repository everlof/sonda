# Pipeline Architecture

> **Keep this diagram up to date.** When adding new input formats, processing
> steps, classification engines, or output formats, update the diagram below
> to reflect the change.

## Overview

```mermaid
flowchart TB
    subgraph Input
        PDF["PDF\n(Eurofins lab report)"]
        XLSX["XLSX\n(Sweco AVFALLSKLASSNING)"]
        JSON["JSON\n(pre-parsed reports)"]
    end

    subgraph Extraction
        PDFTXT["pdftotext -layout\nextraction/pdftotext.rs"]
        XLSXP["calamine reader\nextraction/sweco_xlsx.rs"]
    end

    subgraph Parsing["Parsing (PDF only)"]
        SPLIT["Split sections\n(on 'Analysrapport')"]
        HEADER["Parse header\n(lab, sample, matrix, date)"]
        TABLE["Parse table rows\n(whitespace-gap splitting)"]
        NORM["Normalize substance\nparsing/normalize.rs"]
        VALP["Parse value\nparsing/values.rs"]
    end

    subgraph Types["Core Types (model.rs)"]
        PR["ParsedReports\n{ reports, warnings, skipped_lines }"]
        AR["AnalysisReport\n{ header: ReportHeader, rows: Vec‹AnalysisRow› }"]
    end

    subgraph Classification
        FILTER["Filter rulesets\nby matrix (Jord/Asfalt)"]
        THRESH["Threshold engine\nclassify/engine.rs\n(NV/Asfalt rulesets)"]
        HP["HP engine\nclassify/hp_engine.rs\n(EU 1357/2014)"]
        CLP["CLP speciation\nclp/speciation.rs\n+ clp/database.rs"]
    end

    subgraph Rulesets
        NV["nv\n(Naturvårdsverket)"]
        ASF["asfalt\n(Asfalt rulesets)"]
        FA["fa\n(HP/Farligt avfall)"]
        CUSTOM["Custom JSON\nrules"]
    end

    subgraph Output
        TBLOUT["Table\n(human-readable)"]
        JSONOUT["JSON\n(ClassificationResult)"]
    end

    %% Extraction paths
    PDF --> PDFTXT
    XLSX --> XLSXP
    PDFTXT -- "Vec‹PageContent›" --> SPLIT

    %% Parsing flow (PDF)
    SPLIT --> HEADER
    SPLIT --> TABLE
    TABLE --> NORM
    TABLE --> VALP
    HEADER --> AR
    NORM --> AR
    VALP --> AR
    AR --> PR

    %% XLSX produces ParsedReports directly
    XLSXP -- "ParsedReports\n(skips PDF parsing)" --> PR

    %% JSON bypass
    JSON -- "Vec‹AnalysisReport›\n(serde deserialize)" --> AR

    %% Two-step workflow: parse outputs JSON
    PR -. "sonda parse -o json\n(save & inspect)" .-> JSON

    %% Classification
    PR -- "reports" --> FILTER
    AR --> FILTER
    FILTER --> THRESH
    FILTER --> HP
    HP --> CLP

    %% Rulesets feed classification
    NV --> THRESH
    ASF --> THRESH
    CUSTOM --> THRESH
    FA --> HP

    %% Results
    THRESH -- "RuleSetResult\n(category + reasons)" --> TBLOUT
    THRESH -- "RuleSetResult" --> JSONOUT
    HP -- "RuleSetResult\n(+ HpDetails)" --> TBLOUT
    HP -- "RuleSetResult\n(+ HpDetails)" --> JSONOUT
```

## CLI Commands

| Command | Input | Pipeline | Output |
|---------|-------|----------|--------|
| `sonda parse report.pdf` | PDF | Extract → Parse | Table or JSON |
| `sonda parse sweco.xlsx` | XLSX | XLSX Parse | Table or JSON |
| `sonda classify report.pdf` | PDF | Extract → Parse → Classify | Table or JSON |
| `sonda classify sweco.xlsx` | XLSX | XLSX Parse → Classify | Table or JSON |
| `sonda classify parsed.json` | JSON | Deserialize → Classify | Table or JSON |

## Key Data Types

```
PDF bytes
  → Vec<PageContent> { lines, line_spans }        (extraction)
    → ParsedReports { reports, warnings, skipped } (parsing)
      → Vec<AnalysisReport> { header, rows }       (core model)
        → ClassificationResult { samples, trace }  (classification)
          → Vec<SampleResult> { ruleset_results }
            → RuleSetResult { category, substances, hp_details }
```
