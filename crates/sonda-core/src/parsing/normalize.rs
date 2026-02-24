use std::collections::HashMap;
use std::sync::LazyLock;

/// Normalize a substance name from a lab report to a canonical key.
///
/// Steps:
/// 1. Lowercase
/// 2. Remove chemical symbol suffixes like " (As)", " As"
/// 3. Replace spaces, hyphens, parentheses with underscores
/// 4. Collapse multiple underscores
/// 5. Look up in alias map
pub fn normalize_substance(raw: &str) -> String {
    let mut s = raw.trim().to_string();

    // Remove trailing chemical symbols in parentheses: "Arsenik (As)" -> "Arsenik"
    // Must be done BEFORE lowercasing to detect uppercase chemical symbols
    if let Some(idx) = s.rfind('(') {
        let after = &s[idx..];
        // Only strip if it looks like a chemical symbol, e.g., "(As)", "(Pb)"
        if after.len() <= 6 && after.ends_with(')') {
            s = s[..idx].trim_end().to_string();
        }
    }

    // Remove trailing chemical symbol without parens: "Arsenik As" -> "Arsenik"
    // Only strip known element symbols to avoid false positives (e.g., "PAH L", "PAH H")
    let words: Vec<&str> = s.split_whitespace().collect();
    if words.len() >= 2 {
        let last = words[words.len() - 1];
        if is_element_symbol(last) {
            s = words[..words.len() - 1].join(" ");
        }
    }

    // Now lowercase
    s = s.to_lowercase();

    // Normalize separators
    let mut normalized = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'a'..='z' | 'å' | 'ä' | 'ö' | '0'..='9' => normalized.push(c),
            ' ' | '-' | '(' | ')' | '/' | ',' | '+' => normalized.push('_'),
            _ => normalized.push('_'),
        }
    }

    // Collapse multiple underscores and trim
    let mut result = String::with_capacity(normalized.len());
    let mut prev_underscore = true; // start true to skip leading underscores
    for c in normalized.chars() {
        if c == '_' {
            if !prev_underscore {
                result.push('_');
            }
            prev_underscore = true;
        } else {
            result.push(c);
            prev_underscore = false;
        }
    }
    // Trim trailing underscore
    if result.ends_with('_') {
        result.pop();
    }

    // Look up alias
    if let Some(canonical) = ALIASES.get(result.as_str()) {
        canonical.to_string()
    } else {
        result
    }
}

/// Check if a string is a known chemical element symbol relevant to environmental analysis.
fn is_element_symbol(s: &str) -> bool {
    matches!(
        s,
        "As" | "Ba"
            | "Pb"
            | "Cd"
            | "Co"
            | "Cu"
            | "Cr"
            | "Hg"
            | "Ni"
            | "V"
            | "Zn"
            | "Fe"
            | "Mn"
            | "Mo"
            | "Sb"
            | "Se"
            | "Sn"
            | "Ti"
            | "Tl"
            | "W"
    )
}

static ALIASES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // Metals - common Eurofins naming variations
    m.insert("arsenik", "arsenik");
    m.insert("arsen", "arsenik");
    m.insert("as", "arsenik");
    m.insert("barium", "barium");
    m.insert("ba", "barium");
    m.insert("bly", "bly");
    m.insert("pb", "bly");
    m.insert("kadmium", "kadmium");
    m.insert("cd", "kadmium");
    m.insert("kobolt", "kobolt");
    m.insert("co", "kobolt");
    m.insert("koppar", "koppar");
    m.insert("cu", "koppar");
    m.insert("krom", "krom_total");
    m.insert("krom_total", "krom_total");
    m.insert("krom_totalt", "krom_total");
    m.insert("cr", "krom_total");
    m.insert("kvicksilver", "kvicksilver");
    m.insert("hg", "kvicksilver");
    m.insert("nickel", "nickel");
    m.insert("ni", "nickel");
    m.insert("vanadin", "vanadin");
    m.insert("v", "vanadin");
    m.insert("zink", "zink");
    m.insert("zn", "zink");

    // BTEX
    m.insert("bensen", "bensen");
    m.insert("benzen", "bensen");
    m.insert("toluen", "toluen");
    m.insert("etylbensen", "etylbensen");
    m.insert("xylener", "xylener");
    m.insert("xylen", "xylener");

    // Aliphatics
    m.insert("alifater_c5_c8", "alifater_c5_c8");
    m.insert("alifater_>c5_c8", "alifater_c5_c8");
    m.insert("alifater__c5_c8", "alifater_c5_c8");
    m.insert("alifater_c8_c10", "alifater_c8_c10");
    m.insert("alifater_>c8_c10", "alifater_c8_c10");
    m.insert("alifater__c8_c10", "alifater_c8_c10");
    m.insert("alifater_c10_c12", "alifater_c10_c12");
    m.insert("alifater_>c10_c12", "alifater_c10_c12");
    m.insert("alifater__c10_c12", "alifater_c10_c12");
    m.insert("alifater_c12_c16", "alifater_c12_c16");
    m.insert("alifater_>c12_c16", "alifater_c12_c16");
    m.insert("alifater__c12_c16", "alifater_c12_c16");
    m.insert("alifater_c16_c35", "alifater_c16_c35");
    m.insert("alifater_>c16_c35", "alifater_c16_c35");
    m.insert("alifater__c16_c35", "alifater_c16_c35");

    // Aromatics
    m.insert("aromater_>c8_c10", "aromater_c8_c10");
    m.insert("aromater_c8_c10", "aromater_c8_c10");
    m.insert("aromater__c8_c10", "aromater_c8_c10");
    m.insert("aromater_>c10_c16", "aromater_c10_c16");
    m.insert("aromater_c10_c16", "aromater_c10_c16");
    m.insert("aromater__c10_c16", "aromater_c10_c16");
    m.insert("aromater_>c16_c35", "aromater_c16_c35");
    m.insert("aromater_c16_c35", "aromater_c16_c35");
    m.insert("aromater__c16_c35", "aromater_c16_c35");

    // PAH groups
    m.insert("pah_l", "pah_l");
    m.insert("pah_l_summa", "pah_l");
    m.insert("summa_pah_l", "pah_l");
    m.insert("pah_låg", "pah_l");
    m.insert("summa_pah_med_låg_molekylvikt", "pah_l");
    m.insert("pah_med_låg_molekylvikt", "pah_l");
    m.insert("pah_m", "pah_m");
    m.insert("pah_m_summa", "pah_m");
    m.insert("summa_pah_m", "pah_m");
    m.insert("pah_medel", "pah_m");
    m.insert("summa_pah_med_medelhög_molekylvikt", "pah_m");
    m.insert("pah_med_medelhög_molekylvikt", "pah_m");
    m.insert("pah_h", "pah_h");
    m.insert("pah_h_summa", "pah_h");
    m.insert("summa_pah_h", "pah_h");
    m.insert("pah_hög", "pah_h");
    m.insert("summa_pah_med_hög_molekylvikt", "pah_h");
    m.insert("pah_med_hög_molekylvikt", "pah_h");

    // PAH-16
    m.insert("pah_16", "pah_16");
    m.insert("summa_16_pah", "pah_16");
    m.insert("pah_16_summa", "pah_16");
    m.insert("summa_pah_16", "pah_16");
    m.insert("summa_totala_pah16", "pah_16");

    // Individual PAH compounds
    m.insert("naftalen", "naftalen");
    m.insert("acenaftylen", "acenaftylen");
    m.insert("acenaften", "acenaften");
    m.insert("fluoren", "fluoren");
    m.insert("fenantren", "fenantren");
    m.insert("antracen", "antracen");
    m.insert("fluoranten", "fluoranten");
    m.insert("pyren", "pyren");
    m.insert("benso_a_antracen", "benso_a_antracen");
    m.insert("krysen", "krysen");
    m.insert("benso_b_fluoranten", "benso_b_fluoranten");
    m.insert("benso_k_fluoranten", "benso_k_fluoranten");
    m.insert("benso_b_k_fluoranten", "benso_b_k_fluoranten");
    m.insert("benso_a_pyren", "benso_a_pyren");
    m.insert("dibenso_a_h_antracen", "dibenso_a_h_antracen");
    m.insert("benso_ghi_perylen", "benso_ghi_perylen");
    m.insert("benso_g_h_i_perylen", "benso_ghi_perylen");
    m.insert("indeno_1_2_3_cd_pyren", "indeno_1_2_3_cd_pyren");
    m.insert("indeno_123cd_pyren", "indeno_1_2_3_cd_pyren");
    m.insert("indeno_123_cd_pyren", "indeno_1_2_3_cd_pyren");

    // Dry substance
    m.insert("ts", "ts");
    m.insert("torrsubstans", "ts");
    m.insert("ts_halt", "ts");

    m
});

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_name() {
        assert_eq!(normalize_substance("Bly"), "bly");
    }

    #[test]
    fn test_with_chemical_symbol_parens() {
        assert_eq!(normalize_substance("Arsenik (As)"), "arsenik");
    }

    #[test]
    fn test_with_chemical_symbol_no_parens() {
        assert_eq!(normalize_substance("Arsenik As"), "arsenik");
    }

    #[test]
    fn test_krom_total() {
        assert_eq!(normalize_substance("Krom total"), "krom_total");
        assert_eq!(normalize_substance("Krom totalt"), "krom_total");
    }

    #[test]
    fn test_aliphatic_ranges() {
        assert_eq!(normalize_substance("Alifater >C5-C8"), "alifater_c5_c8");
        assert_eq!(normalize_substance("Alifater C10-C12"), "alifater_c10_c12");
        assert_eq!(normalize_substance("Alifater >C16-C35"), "alifater_c16_c35");
    }

    #[test]
    fn test_pah_groups() {
        assert_eq!(normalize_substance("PAH L summa"), "pah_l");
        assert_eq!(normalize_substance("PAH M summa"), "pah_m");
        assert_eq!(normalize_substance("PAH H summa"), "pah_h");
        assert_eq!(normalize_substance("Summa PAH L"), "pah_l");
    }

    #[test]
    fn test_pah_16() {
        assert_eq!(normalize_substance("Summa 16 PAH"), "pah_16");
        assert_eq!(normalize_substance("PAH 16 summa"), "pah_16");
    }

    #[test]
    fn test_individual_pah() {
        assert_eq!(normalize_substance("Benso(a)pyren"), "benso_a_pyren");
        assert_eq!(
            normalize_substance("Indeno(1,2,3-cd)pyren"),
            "indeno_1_2_3_cd_pyren"
        );
    }

    #[test]
    fn test_whitespace_handling() {
        assert_eq!(normalize_substance("  Bly  "), "bly");
    }

    #[test]
    fn test_unknown_substance_passthrough() {
        assert_eq!(
            normalize_substance("Unknown Substance"),
            "unknown_substance"
        );
    }
}
