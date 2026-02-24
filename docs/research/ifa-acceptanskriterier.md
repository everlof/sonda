# IFA Acceptanskriterier

## Status: Phase 2 (Deferred)

## Source

- NFS 2004:10 (Naturvårdsverkets föreskrifter om deponering, kriterier och förfaranden för mottagning av avfall vid anläggningar för deponering av avfall)
- Sections 22-23: Acceptanskriterier for inert waste (IFA = Inert Waste Landfill)
- Based on EU Council Decision 2003/33/EC

## Key Concepts

IFA (Inert Fyllnadsavfall) criteria determine whether waste can be disposed of at an inert waste landfill. The criteria include:

1. **Total content values** (mg/kg TS): Maximum allowed concentrations in the waste material
2. **Leaching values** (L/S=10, mg/kg): Maximum allowed leaching at liquid-to-solid ratio of 10 L/kg

## Parameter Thresholds

### Total Content (mg/kg TS)

These are the parameters relevant to soil and construction waste:

| Parameter | IFA Limit |
|---|---|
| TOC (Total Organic Carbon) | 30 000 |
| BTEX | 6 |
| PCB (7 congeners) | 1 |
| Mineral oil (C10-C40) | 500 |
| PAH (sum) | 100* |

*Note: The PAH limit interpretation varies; some references use individual compound limits.

### Leaching Values (L/S=10, mg/l or mg/kg)

| Parameter | IFA Limit (mg/kg) |
|---|---|
| As | 0.5 |
| Ba | 20 |
| Cd | 0.04 |
| Cr total | 0.5 |
| Cu | 2 |
| Hg | 0.01 |
| Mo | 0.5 |
| Ni | 0.4 |
| Pb | 0.5 |
| Sb | 0.06 |
| Se | 0.1 |
| Zn | 4 |
| Chloride | 800 |
| Fluoride | 10 |
| Sulfate | 1000 |
| DOC | 500 |
| TDS | 4000 |
| pH | 6-13 (range) |

## Implementation Notes

IFA classification requires both total content AND leaching values. Most Eurofins reports include total content analysis, but leaching tests are separate analyses. The Phase 2 implementation should:

1. Support both total content and leaching value rules in the JSON schema
2. Clearly indicate when leaching data is missing from a report
3. Handle the pH range criterion (not a simple threshold)
