use sonda_core::rules::builtin;
use std::path::Path;

pub fn list() -> Result<(), sonda_core::error::SondaError> {
    println!("Available predefined rulesets:\n");
    for name in builtin::PRESETS {
        if builtin::is_hp_preset(name) {
            println!("  {:<8} Farligt avfall (HP-bedömning)", name);
            println!("           CLP/HP-based hazardous waste classification per EU Regulation 1357/2014");
            println!();
        } else {
            let rs = builtin::load_preset(name)?;
            let matrix_info = match rs.matrix.as_deref() {
                Some(m) => format!(" [{}]", m),
                None => String::new(),
            };
            println!("  {:<8} {} (v{}){}", name, rs.name, rs.version, matrix_info);
            if let Some(ref desc) = rs.description {
                println!("           {}", desc);
            }
            println!();
        }
    }
    Ok(())
}

pub fn explain(preset: &str) -> Result<(), sonda_core::error::SondaError> {
    if builtin::is_hp_preset(preset) {
        return explain_fa();
    }

    let rs = builtin::load_preset(preset)?;

    println!("{} (version {})\n", rs.name, rs.version);

    if let Some(ref desc) = rs.description {
        println!("{}\n", desc);
    }

    let material = match rs.matrix.as_deref() {
        Some("jord") => "soil",
        Some("asfalt") => "asphalt",
        _ => "samples",
    };
    println!(
        "This ruleset classifies {} into {} categories:\n",
        material,
        rs.categories.len()
    );

    for cat in &rs.categories {
        print!("  {}", cat);
        if let Some(desc) = rs.category_descriptions.get(cat) {
            println!(" -- {}", desc);
        } else {
            println!();
        }
        println!();
    }

    println!("If any substance exceeds all thresholds, the soil does not meet");
    println!("either standard and requires further assessment.\n");

    // Print thresholds table
    println!("Thresholds:");
    println!();

    // Find max substance name length for alignment
    let max_name_len = rs
        .rules
        .iter()
        .map(|r| r.substance.len())
        .max()
        .unwrap_or(20);

    // Header
    print!("  {:<width$}", "Substance", width = max_name_len + 2);
    for cat in &rs.categories {
        print!("  {:<10}", cat);
    }
    println!("  Unit");
    println!(
        "  {}",
        "-".repeat(max_name_len + 2 + (rs.categories.len() * 12) + 10)
    );

    for rule in &rs.rules {
        print!("  {:<width$}", rule.substance, width = max_name_len + 2);
        for cat in &rs.categories {
            if let Some(threshold) = rule.thresholds.get(cat) {
                print!("  {:<10}", threshold);
            } else {
                print!("  {:<10}", "-");
            }
        }
        print!("  {}", rule.unit.as_deref().unwrap_or("mg/kg TS"));
        println!();
    }

    println!();

    Ok(())
}

fn explain_fa() -> Result<(), sonda_core::error::SondaError> {
    println!("Farligt avfall (HP-bedömning)\n");
    println!("CLP/HP-based hazardous waste classification per EU Regulation 1357/2014");
    println!("and Commission Regulation 2017/997 (HP14 ecotoxic).\n");
    println!("This preset evaluates waste against the Hazardous Properties (HP) criteria.");
    println!("Result is binary: FA (hazardous waste) or Icke FA (non-hazardous waste).\n");
    println!("Evaluated HP criteria:\n");
    println!("  HP4   Irritant              Summation: H315, H319 >= 20%");
    println!("  HP5   STOT SE/RE            Individual: H370 >= 1%, H371 >= 10%");
    println!("                              Summation: H372 >= 1%, H373 >= 10%");
    println!("  HP6   Acute Toxicity        Summation per route and category");
    println!("  HP7   Carcinogenic          Individual: H350 >= 0.1%, H351 >= 1%");
    println!("  HP8   Corrosive             Summation: H314 >= 5%");
    println!("  HP10  Toxic for repro.      Individual: H360 >= 0.3% (SCL: Pb 0.03%)");
    println!("                              Individual: H361 >= 0.3%");
    println!("  HP11  Mutagenic             Individual: H340 >= 0.1%, H341 >= 1%");
    println!("  HP13  Sensitising           Individual: H317/H334 >= 10%");
    println!("  HP14  Ecotoxic              Multiple summation checks with M-factors\n");
    println!("Speciation: metals are converted to worst-case CLP compounds using");
    println!("molecular weight conversion factors (e.g., As -> As2O3 x 1.32).");
    println!("Concentrations are converted from mg/kg TS to % w/w (divide by 10000).\n");
    println!("Below-detection values contribute 0 to summation checks.\n");

    Ok(())
}

pub fn schema() -> Result<(), sonda_core::error::SondaError> {
    print!(
        r#"JSON Rule Schema
================

A rule file defines a set of classification thresholds for chemical
substances. When you run `sonda classify`, each substance in the lab
report is compared against these thresholds to determine its category.

Top-level fields:
  name          (string, required)  Human-readable name of the ruleset
  description   (string, optional)  What this ruleset is for
  version       (string, required)  Version identifier (e.g., "2025.1")
  categories    (array, required)   Ordered list of category names,
                                    from cleanest to most contaminated.
                                    Example: ["KM", "MKM"]
  category_descriptions
                (object, optional)  Map of category name to human-readable
                                    description. Used by `sonda rules explain`.
  rules         (array, required)   List of substance rules (see below)

Each rule in the "rules" array:
  substance     (string, required)  Canonical substance name (lowercase).
                                    Must match sonda's normalized names.
                                    Run `sonda rules explain <preset>` to see
                                    all recognized substance keys.
  thresholds    (object, required)  Map of category -> threshold value.
                                    Values are strings representing decimal
                                    numbers (e.g., "10", "0.25", "2.5").
                                    A substance is classified into the first
                                    category whose threshold it does NOT exceed.
  matrix        (string, optional)  Only apply this rule when the sample matrix
                                    matches. Values: "jord" or "asfalt".
                                    Omit to apply regardless of matrix.
  unit          (string, optional)  Unit for display. Default: "mg/kg TS"
  note          (string, optional)  Regulatory reference or explanation.

Example:
{{
  "name": "My custom ruleset",
  "description": "Project-specific thresholds for Site X",
  "version": "1.0",
  "categories": ["Clean", "Moderate", "Contaminated"],
  "category_descriptions": {{
    "Clean": "Below background levels, free reuse",
    "Moderate": "Acceptable for industrial use with cover",
    "Contaminated": "Requires remediation or controlled disposal"
  }},
  "rules": [
    {{
      "substance": "bly",
      "thresholds": {{
        "Clean": "20",
        "Moderate": "100",
        "Contaminated": "500"
      }},
      "unit": "mg/kg TS",
      "note": "Site-specific based on risk assessment"
    }},
    {{
      "substance": "pah_16",
      "matrix": "asfalt",
      "thresholds": {{
        "Clean": "70",
        "Contaminated": "300"
      }}
    }}
  ]
}}

Note: threshold values must be quoted strings, not bare numbers,
to preserve exact decimal precision (e.g., "0.25" not 0.25).

The "fa" preset uses the HP engine (not JSON thresholds). Use
`sonda rules explain fa` for HP criteria details.
"#
    );
    Ok(())
}

pub fn validate(file: &Path) -> Result<(), sonda_core::error::SondaError> {
    let rs = sonda_core::rules::load_ruleset(file)?;

    println!("Ruleset '{}' (v{}) is valid.", rs.name, rs.version);
    println!("  Categories: {}", rs.categories.join(", "));
    println!("  Rules: {} substances", rs.rules.len());

    // Check for potential issues (warnings, not errors)
    let mut warnings = Vec::new();
    for rule in &rs.rules {
        // Warn if not all categories have thresholds
        for cat in &rs.categories {
            if !rule.thresholds.contains_key(cat) {
                warnings.push(format!(
                    "substance '{}' has no threshold for category '{}'",
                    rule.substance, cat
                ));
            }
        }
    }

    if !warnings.is_empty() {
        println!("\nWarnings:");
        for w in &warnings {
            println!("  - {}", w);
        }
    }

    Ok(())
}
