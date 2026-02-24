pub mod builtin;
pub mod schema;

use crate::error::SondaError;
use schema::RuleSetDef;
use std::path::Path;

/// Load a ruleset from a JSON file.
pub fn load_ruleset(path: &Path) -> Result<RuleSetDef, SondaError> {
    let content = std::fs::read_to_string(path).map_err(|e| SondaError::RulesetLoad {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    parse_ruleset(&content, path)
}

/// Parse a ruleset from a JSON string.
pub fn parse_ruleset(json: &str, source: &Path) -> Result<RuleSetDef, SondaError> {
    let ruleset: RuleSetDef = serde_json::from_str(json).map_err(|e| SondaError::RulesetLoad {
        path: source.to_path_buf(),
        reason: e.to_string(),
    })?;
    validate_ruleset(&ruleset)?;
    Ok(ruleset)
}

/// Parse a ruleset from a JSON string (no file path context).
pub fn parse_ruleset_str(json: &str) -> Result<RuleSetDef, SondaError> {
    let ruleset: RuleSetDef = serde_json::from_str(json).map_err(SondaError::Json)?;
    validate_ruleset(&ruleset)?;
    Ok(ruleset)
}

/// Validate that a ruleset is well-formed.
pub fn validate_ruleset(ruleset: &RuleSetDef) -> Result<(), SondaError> {
    if let Some(ref matrix) = ruleset.matrix {
        let lower = matrix.to_lowercase();
        if lower != "jord" && lower != "asfalt" {
            return Err(SondaError::RulesetInvalid(format!(
                "invalid top-level matrix '{}' (expected 'jord' or 'asfalt')",
                matrix
            )));
        }
    }

    if ruleset.categories.is_empty() {
        return Err(SondaError::RulesetInvalid(
            "categories must not be empty".into(),
        ));
    }

    if ruleset.rules.is_empty() {
        return Err(SondaError::RulesetInvalid("rules must not be empty".into()));
    }

    for rule in &ruleset.rules {
        if rule.substance.is_empty() {
            return Err(SondaError::RulesetInvalid(
                "substance name must not be empty".into(),
            ));
        }

        if rule.thresholds.is_empty() {
            return Err(SondaError::RulesetInvalid(format!(
                "substance '{}' has no thresholds",
                rule.substance
            )));
        }

        for cat in rule.thresholds.keys() {
            if !ruleset.categories.contains(cat) {
                return Err(SondaError::RulesetInvalid(format!(
                    "substance '{}' references unknown category '{}'",
                    rule.substance, cat
                )));
            }
        }

        if let Some(ref matrix) = rule.matrix {
            let lower = matrix.to_lowercase();
            if lower != "jord" && lower != "asfalt" {
                return Err(SondaError::RulesetInvalid(format!(
                    "substance '{}' has invalid matrix '{}' (expected 'jord' or 'asfalt')",
                    rule.substance, matrix
                )));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_ruleset() {
        let json = r#"{
            "name": "Test",
            "version": "1.0",
            "categories": ["A", "B"],
            "rules": [
                { "substance": "bly", "thresholds": { "A": "50", "B": "180" } }
            ]
        }"#;
        let rs = parse_ruleset_str(json).unwrap();
        assert_eq!(rs.name, "Test");
        assert_eq!(rs.categories, vec!["A", "B"]);
        assert_eq!(rs.rules.len(), 1);
    }

    #[test]
    fn test_empty_categories_rejected() {
        let json = r#"{
            "name": "Bad",
            "version": "1.0",
            "categories": [],
            "rules": [
                { "substance": "bly", "thresholds": { "A": "50" } }
            ]
        }"#;
        assert!(parse_ruleset_str(json).is_err());
    }

    #[test]
    fn test_unknown_category_in_threshold_rejected() {
        let json = r#"{
            "name": "Bad",
            "version": "1.0",
            "categories": ["A"],
            "rules": [
                { "substance": "bly", "thresholds": { "X": "50" } }
            ]
        }"#;
        assert!(parse_ruleset_str(json).is_err());
    }

    #[test]
    fn test_invalid_matrix_rejected() {
        let json = r#"{
            "name": "Bad",
            "version": "1.0",
            "categories": ["A"],
            "rules": [
                { "substance": "bly", "thresholds": { "A": "50" }, "matrix": "vatten" }
            ]
        }"#;
        assert!(parse_ruleset_str(json).is_err());
    }
}
