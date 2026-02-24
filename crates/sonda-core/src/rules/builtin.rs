use crate::error::SondaError;
use crate::rules::schema::RuleSetDef;

const NV_RIKTVARDEN_JSON: &str = include_str!("../../../../rules/nv-riktvarden.json");
const ASFALT_PAH16_JSON: &str = include_str!("../../../../rules/asfalt-pah16.json");

/// Available predefined rulesets.
pub const PRESETS: &[&str] = &["nv", "asfalt"];

/// Load a predefined ruleset by name.
pub fn load_preset(name: &str) -> Result<RuleSetDef, SondaError> {
    match name {
        "nv" => {
            let ruleset: RuleSetDef = serde_json::from_str(NV_RIKTVARDEN_JSON)?;
            Ok(ruleset)
        }
        "asfalt" => {
            let ruleset: RuleSetDef = serde_json::from_str(ASFALT_PAH16_JSON)?;
            Ok(ruleset)
        }
        _ => Err(SondaError::RulesetInvalid(format!(
            "unknown preset '{}'. Available: {}",
            name,
            PRESETS.join(", ")
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_nv_preset() {
        let rs = load_preset("nv").unwrap();
        assert_eq!(rs.categories, vec!["KM", "MKM"]);
        assert!(!rs.rules.is_empty());
    }

    #[test]
    fn test_unknown_preset() {
        assert!(load_preset("xyz").is_err());
    }
}
