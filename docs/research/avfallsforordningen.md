# Avfallsförordningen -- FA Classification

## Status: Phase 2 (Deferred)

## Source

- EU regulation 1357/2014 (HP criteria for hazardous waste)
- Swedish Avfallsförordningen (SFS 2020:614)
- Naturvårdsverket guidance on waste classification

## Key Concepts

FA (Farligt Avfall / Hazardous Waste) classification is fundamentally different from the NV KM/MKM system:

1. **HP criteria system**: EU defines 15 hazardous properties (HP1-HP15). Waste is classified as hazardous if it meets ANY HP criterion.
2. **Not simple threshold exceedance**: Unlike KM/MKM, FA requires evaluating multiple HP properties, each with its own calculation method.
3. **Concentration-based**: Uses total concentration and sometimes leaching values.

## Relevant HP Criteria for Soil/Asphalt

- **HP7 (Carcinogenic)**: E.g., BaP > certain limits
- **HP14 (Ecotoxic)**: Sum calculations with M-factors
- **HP4-HP6, HP8 (Toxicity)**: Various toxicity endpoints

## Practical Screening Thresholds

In practice, Swedish environmental consultants often use simplified screening thresholds:

### Asphalt
- BaP > 50 mg/kg → likely hazardous waste
- PAH-16 total content used as indicator
- Waste codes: 17 03 01* (hazardous) / 17 03 02 (non-hazardous)

### Soil
- Metal concentrations evaluated against HP criteria
- Waste codes: 17 05 03* (hazardous) / 17 05 04 (non-hazardous)

## Implementation Notes

FA classification will be implemented as a separate ruleset in Phase 2. The implementation must clearly communicate that the screening thresholds are simplified and that full HP assessment may be required for borderline cases.
