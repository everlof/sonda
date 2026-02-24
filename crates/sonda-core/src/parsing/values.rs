use crate::error::SondaError;
use crate::model::AnalysisValue;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Parse a value string from a lab report into an AnalysisValue.
///
/// Handles formats like:
/// - "68" -> Measured(68)
/// - "0.030" -> Measured(0.030)
/// - "< 0.030" -> BelowDetection(0.030)
/// - "<0.030" -> BelowDetection(0.030)
/// - "< 0,030" -> BelowDetection(0.030) (Swedish decimal comma)
/// - "*" or similar markers are ignored (returns None)
pub fn parse_value(s: &str) -> Result<Option<AnalysisValue>, SondaError> {
    let s = s.trim();

    if s.is_empty() || s == "*" || s == "-" || s == "—" || s == "n.a." || s == "N/A" {
        return Ok(None);
    }

    // Check for below-detection-limit marker
    if let Some(rest) = s.strip_prefix('<') {
        let rest = rest.trim();
        let decimal = parse_decimal(rest)?;
        return Ok(Some(AnalysisValue::BelowDetection(decimal)));
    }

    let decimal = parse_decimal(s)?;
    Ok(Some(AnalysisValue::Measured(decimal)))
}

/// Parse a decimal value, handling Swedish comma notation.
fn parse_decimal(s: &str) -> Result<Decimal, SondaError> {
    let s = s.trim();
    // Replace Swedish decimal comma with dot
    let normalized = s.replace(',', ".");
    Decimal::from_str(&normalized)
        .map_err(|e| SondaError::ParseError(format!("invalid number '{}': {}", s, e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_measured_integer() {
        let v = parse_value("68").unwrap().unwrap();
        assert_eq!(v, AnalysisValue::Measured(dec!(68)));
    }

    #[test]
    fn test_measured_decimal() {
        let v = parse_value("0.030").unwrap().unwrap();
        assert_eq!(v, AnalysisValue::Measured(dec!(0.030)));
    }

    #[test]
    fn test_below_detection_with_space() {
        let v = parse_value("< 0.030").unwrap().unwrap();
        assert_eq!(v, AnalysisValue::BelowDetection(dec!(0.030)));
    }

    #[test]
    fn test_below_detection_no_space() {
        let v = parse_value("<0.030").unwrap().unwrap();
        assert_eq!(v, AnalysisValue::BelowDetection(dec!(0.030)));
    }

    #[test]
    fn test_swedish_comma() {
        let v = parse_value("0,030").unwrap().unwrap();
        assert_eq!(v, AnalysisValue::Measured(dec!(0.030)));
    }

    #[test]
    fn test_below_detection_swedish_comma() {
        let v = parse_value("< 0,030").unwrap().unwrap();
        assert_eq!(v, AnalysisValue::BelowDetection(dec!(0.030)));
    }

    #[test]
    fn test_whitespace_trimming() {
        let v = parse_value("  68  ").unwrap().unwrap();
        assert_eq!(v, AnalysisValue::Measured(dec!(68)));
    }

    #[test]
    fn test_empty_returns_none() {
        assert!(parse_value("").unwrap().is_none());
    }

    #[test]
    fn test_star_returns_none() {
        assert!(parse_value("*").unwrap().is_none());
    }

    #[test]
    fn test_dash_returns_none() {
        assert!(parse_value("-").unwrap().is_none());
    }

    #[test]
    fn test_invalid_returns_error() {
        assert!(parse_value("abc").is_err());
    }
}
