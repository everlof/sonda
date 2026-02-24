use super::schema::{ClpDatabase, ClpSubstance, SpeciationTable};
use std::sync::LazyLock;

const CLP_SUBSTANCES_JSON: &str = include_str!("../../../../rules/clp-substances.json");
const SPECIATION_JSON: &str = include_str!("../../../../rules/speciation-assumptions.json");

static CLP_DATABASE: LazyLock<ClpDatabase> = LazyLock::new(|| {
    serde_json::from_str(CLP_SUBSTANCES_JSON).expect("embedded clp-substances.json is valid")
});

static SPECIATION_TABLE: LazyLock<SpeciationTable> = LazyLock::new(|| {
    serde_json::from_str(SPECIATION_JSON).expect("embedded speciation-assumptions.json is valid")
});

/// Get the CLP substance database.
pub fn clp_database() -> &'static ClpDatabase {
    &CLP_DATABASE
}

/// Get the speciation assumptions table.
pub fn speciation_table() -> &'static SpeciationTable {
    &SPECIATION_TABLE
}

/// Look up a CLP substance by CAS number.
pub fn lookup_by_cas(cas: &str) -> Option<&'static ClpSubstance> {
    CLP_DATABASE.substances.get(cas)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clp_database_loads() {
        let db = clp_database();
        assert!(!db.substances.is_empty());
        assert!(db.substances.contains_key("1327-53-3")); // As2O3
        assert!(db.substances.contains_key("50-32-8")); // BaP
    }

    #[test]
    fn test_speciation_table_loads() {
        let st = speciation_table();
        assert!(!st.metals.is_empty());
        assert!(!st.pah_direct.is_empty());
    }

    #[test]
    fn test_lookup_arsenic() {
        let sub = lookup_by_cas("1327-53-3").unwrap();
        assert!(sub.name.contains("As2O3"));
        assert!(sub.has_h_code("H350"));
        assert!(sub.has_h_code("H301"));
        assert!(sub.has_h_code("H410"));
    }

    #[test]
    fn test_lookup_bap() {
        let sub = lookup_by_cas("50-32-8").unwrap();
        assert!(sub.name.contains("Benso(a)pyren"));
        assert!(sub.has_h_code("H350"));
        assert!(sub.has_h_code("H340"));
        assert!(sub.has_h_code("H360FD"));
    }

    #[test]
    fn test_copper_m_factors() {
        let sub = lookup_by_cas("1317-39-1").unwrap();
        assert_eq!(sub.m_factors.acute, Some(rust_decimal_macros::dec!(100)));
        assert_eq!(sub.m_factors.chronic, Some(rust_decimal_macros::dec!(1)));
    }

    #[test]
    fn test_lead_scl() {
        let sub = lookup_by_cas("7439-92-1").unwrap();
        assert!(sub.scls.contains_key("Repr.1A"));
        assert_eq!(
            sub.scls.get("Repr.1A"),
            Some(&rust_decimal_macros::dec!(0.03))
        );
    }

    #[test]
    fn test_speciation_metals_complete() {
        let st = speciation_table();
        let metal_names: Vec<&str> = st.metals.iter().map(|m| m.substance.as_str()).collect();
        assert!(metal_names.contains(&"arsenik"));
        assert!(metal_names.contains(&"kadmium"));
        assert!(metal_names.contains(&"krom_total"));
        assert!(metal_names.contains(&"koppar"));
        assert!(metal_names.contains(&"bly"));
        assert!(metal_names.contains(&"kvicksilver"));
        assert!(metal_names.contains(&"nickel"));
        assert!(metal_names.contains(&"zink"));
        assert!(metal_names.contains(&"barium"));
        assert!(metal_names.contains(&"kobolt"));
        assert!(metal_names.contains(&"vanadin"));
    }

    #[test]
    fn test_speciation_pah_direct() {
        let st = speciation_table();
        let pah_names: Vec<&str> = st.pah_direct.iter().map(|p| p.substance.as_str()).collect();
        assert!(pah_names.contains(&"benso_a_pyren"));
        assert!(pah_names.contains(&"naftalen"));
        assert!(pah_names.contains(&"antracen"));
    }

    #[test]
    fn test_all_speciation_cas_exist_in_clp() {
        let st = speciation_table();
        let db = clp_database();
        for metal in &st.metals {
            assert!(
                db.substances.contains_key(&metal.cas),
                "CAS {} for {} not found in CLP database",
                metal.cas,
                metal.substance
            );
        }
        for pah in &st.pah_direct {
            assert!(
                db.substances.contains_key(&pah.cas),
                "CAS {} for {} not found in CLP database",
                pah.cas,
                pah.substance
            );
        }
    }
}
