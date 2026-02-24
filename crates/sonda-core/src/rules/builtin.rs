use crate::error::SondaError;
use crate::rules::schema::RuleSetDef;

const NV_RIKTVARDEN_JSON: &str = include_str!("../../../../rules/nv-riktvarden.json");
const ASFALT_PAH16_JSON: &str = include_str!("../../../../rules/asfalt-pah16.json");

/// Available predefined rulesets (including HP-based "fa").
pub const PRESETS: &[&str] = &["nv", "asfalt", "fa"];

/// Check if a preset uses the HP engine rather than threshold comparison.
pub fn is_hp_preset(name: &str) -> bool {
    name == "fa"
}

/// Load a predefined ruleset by name.
///
/// Note: "fa" is an HP-based preset and does not return a `RuleSetDef`.
/// Use `is_hp_preset()` to check before calling this function.
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
        "fa" => Err(SondaError::RulesetInvalid(
            "'fa' is an HP-based preset. Use ClassifyOptions.include_hp instead of loading as a ruleset.".into(),
        )),
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

    #[test]
    fn test_fa_is_hp_preset() {
        assert!(is_hp_preset("fa"));
        assert!(!is_hp_preset("nv"));
        assert!(!is_hp_preset("asfalt"));
    }

    #[test]
    fn test_fa_load_returns_error() {
        assert!(load_preset("fa").is_err());
    }
}
