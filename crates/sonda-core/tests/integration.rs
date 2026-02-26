//! Integration tests for classify_pdf() end-to-end pipeline.
//!
//! Uses a MockExtractor that returns pre-built PageContent without
//! invoking pdftotext, so these tests run without poppler-utils.

use sonda_core::classify_pdf;
use sonda_core::error::SondaError;
use sonda_core::extraction::{PageContent, PdfExtractor};
use sonda_core::rules::builtin::load_preset;
use sonda_core::ClassifyOptions;

struct MockExtractor {
    pages: Vec<PageContent>,
}

impl PdfExtractor for MockExtractor {
    fn extract_pages(&self, _pdf_bytes: &[u8]) -> Result<Vec<PageContent>, SondaError> {
        Ok(self.pages.clone())
    }

    fn backend_name(&self) -> &str {
        "mock"
    }
}

fn page(number: usize, lines: &[&str]) -> PageContent {
    PageContent {
        page_number: number,
        lines: lines.iter().map(|s| s.to_string()).collect(),
        line_spans: vec![],
    }
}

// ---------------------------------------------------------------------------
// Test 1: Single soil sample, all metals below KM thresholds
// ---------------------------------------------------------------------------
#[test]
fn single_sample_soil_below_km() {
    let nv = load_preset("nv").unwrap();
    let extractor = MockExtractor {
        pages: vec![page(
            1,
            &[
                "Eurofins Environment Testing Sweden AB",
                "Analysrapport",
                "Rapport: AR-2024-001",
                "Provnummer: P001",
                "Matris: Jord",
                "",
                "  Analys                Resultat    Enhet",
                "  Arsenik (As)          8           mg/kg TS",
                "  Bly (Pb)              45          mg/kg TS",
                "  Kvicksilver (Hg)      < 0.030     mg/kg TS",
                "  Koppar (Cu)           60          mg/kg TS",
            ],
        )],
    };

    let result = classify_pdf(&[], &extractor, &[nv], &ClassifyOptions::default()).unwrap();

    assert_eq!(result.samples.len(), 1);
    let rs = &result.samples[0].ruleset_results[0];
    // As 8 < KM 10, Pb 45 < KM 50, Hg <0.03 < KM 0.25, Cu 60 < KM 80
    assert_eq!(rs.overall_category, "KM");
}

// ---------------------------------------------------------------------------
// Test 2: Asphalt sample with PAH-16 in "Förorenad" range
// ---------------------------------------------------------------------------
#[test]
fn single_sample_asfalt_pah16() {
    let asfalt = load_preset("asfalt").unwrap();
    let extractor = MockExtractor {
        pages: vec![page(
            1,
            &[
                "Eurofins Environment Testing Sweden AB",
                "Analysrapport",
                "Rapport: AR-2024-002",
                "Provnummer: A001",
                "Matris: Asfalt",
                "",
                "  Analys                Resultat    Enhet",
                "  PAH 16               250         mg/kg TS",
            ],
        )],
    };

    let result = classify_pdf(&[], &extractor, &[asfalt], &ClassifyOptions::default()).unwrap();

    assert_eq!(result.samples.len(), 1);
    let rs = &result.samples[0].ruleset_results[0];
    // PAH-16 250: > Ren(70), <= Förorenad(300)
    assert_eq!(rs.overall_category, "Förorenad");
}

// ---------------------------------------------------------------------------
// Test 3: Multi-sample PDF — 2 sections classified independently
// ---------------------------------------------------------------------------
#[test]
fn multi_sample_classified_independently() {
    let nv = load_preset("nv").unwrap();
    let extractor = MockExtractor {
        pages: vec![
            page(
                1,
                &[
                    "Eurofins Environment Testing Sweden AB",
                    "Analysrapport",
                    "Provnummer: S001",
                    "Matris: Jord",
                    "",
                    "  Arsenik (As)          8           mg/kg TS",
                    "  Bly (Pb)              30          mg/kg TS",
                ],
            ),
            page(
                2,
                &[
                    "Analysrapport",
                    "Provnummer: S002",
                    "Matris: Jord",
                    "",
                    "  Arsenik (As)          15          mg/kg TS",
                    "  Bly (Pb)              60          mg/kg TS",
                ],
            ),
        ],
    };

    let result = classify_pdf(&[], &extractor, &[nv], &ClassifyOptions::default()).unwrap();

    assert_eq!(result.samples.len(), 2);
    // S001: As 8 < KM(10), Pb 30 < KM(50) → KM
    assert_eq!(result.samples[0].ruleset_results[0].overall_category, "KM");
    // S002: As 15 > KM(10) but <= MKM(25), Pb 60 > KM(50) but <= MKM(180) → MKM
    assert_eq!(result.samples[1].ruleset_results[0].overall_category, "MKM");
}

// ---------------------------------------------------------------------------
// Test 4: Below-detection with uncertain escalation
// ---------------------------------------------------------------------------
#[test]
fn below_detection_uncertain_escalation() {
    let nv = load_preset("nv").unwrap();
    let extractor = MockExtractor {
        pages: vec![page(
            1,
            &[
                "Eurofins Environment Testing Sweden AB",
                "Analysrapport",
                "Provnummer: BD001",
                "Matris: Jord",
                "",
                // Detection limit 5 < KM(10) → confident KM
                "  Arsenik (As)          < 5         mg/kg TS",
                // Detection limit 0.30 > KM(0.25) → escalated to MKM
                "  Kvicksilver (Hg)      < 0.30      mg/kg TS",
            ],
        )],
    };

    let result = classify_pdf(&[], &extractor, &[nv], &ClassifyOptions::default()).unwrap();

    let rs = &result.samples[0].ruleset_results[0];
    let as_result = rs
        .substance_results
        .iter()
        .find(|r| r.substance == "arsenik")
        .unwrap();
    assert_eq!(as_result.category, "KM");

    let hg_result = rs
        .substance_results
        .iter()
        .find(|r| r.substance == "kvicksilver")
        .unwrap();
    // DL 0.30 > KM(0.25) so escalated beyond KM
    assert_eq!(hg_result.category, "MKM");
}

// ---------------------------------------------------------------------------
// Test 5: HP classification — arsenik triggers FA
// ---------------------------------------------------------------------------
#[test]
fn hp_classification_arsenic_triggers_fa() {
    let extractor = MockExtractor {
        pages: vec![page(
            1,
            &[
                "Eurofins Environment Testing Sweden AB",
                "Analysrapport",
                "Provnummer: HP001",
                "Matris: Jord",
                "",
                // Arsenik 1200 mg/kg × 1.32 MW factor = 1584 mg/kg
                // → 0.1584% w/w >= HP7 H350 threshold 0.1% → triggers
                "  Arsenik (As)          1200        mg/kg TS",
                "  Bly (Pb)              50          mg/kg TS",
            ],
        )],
    };

    let opts = ClassifyOptions { include_hp: true };
    let result = classify_pdf(&[], &extractor, &[], &opts).unwrap();

    assert_eq!(result.samples.len(), 1);
    let hp_rs = result.samples[0]
        .ruleset_results
        .iter()
        .find(|rs| rs.hp_details.is_some())
        .expect("should have HP result");
    assert_eq!(hp_rs.overall_category, "FA");
    assert!(hp_rs.hp_details.as_ref().unwrap().is_hazardous);
}

// ---------------------------------------------------------------------------
// Test 6: Unparseable row generates trace warning
// ---------------------------------------------------------------------------
#[test]
fn unparseable_rows_generate_trace_warnings() {
    let nv = load_preset("nv").unwrap();
    let extractor = MockExtractor {
        pages: vec![page(
            1,
            &[
                "Eurofins Environment Testing Sweden AB",
                "Analysrapport",
                "Provnummer: WARN001",
                "Matris: Jord",
                "",
                "  Arsenik (As)          8           mg/kg TS",
                // "n.d." is not a recognized marker — causes parse_value error
                "  Bly (Pb)              n.d.        mg/kg TS",
                "  Kadmium (Cd)          0.5         mg/kg TS",
            ],
        )],
    };

    let result = classify_pdf(&[], &extractor, &[nv], &ClassifyOptions::default()).unwrap();

    // Classification succeeds with the 2 valid rows
    assert_eq!(result.samples.len(), 1);
    // Trace should contain a warning about the skipped Bly line
    let skipped_warnings: Vec<_> = result
        .trace
        .warnings
        .iter()
        .filter(|w| w.message.contains("Skipped line"))
        .collect();
    assert!(!skipped_warnings.is_empty());
    assert!(skipped_warnings[0].message.contains("Bly"));
}

// ---------------------------------------------------------------------------
// Test 7: Non-Eurofins report returns UnsupportedReport error
// ---------------------------------------------------------------------------
#[test]
fn non_eurofins_report_returns_unsupported_error() {
    let nv = load_preset("nv").unwrap();
    let extractor = MockExtractor {
        pages: vec![page(
            1,
            &[
                "ALS Scandinavia AB",
                "Analysrapport",
                "Provnummer: P001",
                "Matris: Jord",
                "",
                "  Arsenik (As)          8           mg/kg TS",
            ],
        )],
    };

    let result = classify_pdf(&[], &extractor, &[nv], &ClassifyOptions::default());

    assert!(matches!(result, Err(SondaError::UnsupportedReport(_))));
}
