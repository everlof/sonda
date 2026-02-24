# Avfallsförordningen — Farligt avfall (FA) klassificering

## Regulatory Chain

Hazardous waste classification in Sweden follows a chain of EU and national regulations:

1. **CLP Regulation (EC) No 1272/2008** — Classification, Labelling and Packaging of substances and mixtures. Provides the harmonised classification system (H-codes, hazard classes) used as the foundation for waste classification.

2. **Waste Framework Directive 2008/98/EC, Annex III** — Defines 15 hazardous properties (HP1–HP15) that make waste hazardous.

3. **Commission Regulation (EU) No 1357/2014** — Replaces Annex III of the WFD with updated HP criteria, specifying concentration limits and calculation methods for HP4–HP13.

4. **Commission Regulation (EU) 2017/997** — Establishes criteria for HP14 (ecotoxic), the last HP criterion to receive specific evaluation rules.

5. **Swedish Avfallsförordningen (SFS 2020:614)** — National implementation transposing the EU waste classification framework into Swedish law.

## Mirror Entry System

Waste codes in the European List of Waste (LoW) use a mirror entry system where contaminated materials may be classified as hazardous (*) or non-hazardous depending on their composition:

| Material | Hazardous code | Non-hazardous code |
|----------|---------------|-------------------|
| Jord (soil) | 17 05 03* | 17 05 04 |
| Asfalt (asphalt) | 17 03 01* | 17 03 02 |
| Betong (concrete) | 17 01 06* | 17 01 07 |

For mirror entries, the HP criteria must be evaluated to determine which code applies.

## The 15 HP Criteria

| HP | Property | Implemented |
|----|----------|-------------|
| HP1 | Explosive | No — not relevant for soil/asphalt |
| HP2 | Oxidising | No — not relevant for soil/asphalt |
| HP3 | Flammable | No — not relevant for soil/asphalt |
| HP4 | Irritant | Yes |
| HP5 | STOT SE/RE | Yes |
| HP6 | Acute Toxicity | Yes |
| HP7 | Carcinogenic | Yes |
| HP8 | Corrosive | Yes |
| HP9 | Infectious | No — not applicable to chemical analysis |
| HP10 | Toxic for reproduction | Yes |
| HP11 | Mutagenic | Yes |
| HP12 | Release of acute toxic gas | No — not applicable to solid waste analysis |
| HP13 | Sensitising | Yes |
| HP14 | Ecotoxic | Yes |
| HP15 | Capable of exhibiting a hazardous property not directly displayed by the original waste | No — requires case-by-case assessment |

## How Sonda Implements FA Classification

Sonda implements a **full HP engine** (not simplified screening thresholds):

1. **Speciation**: Lab-reported elemental metals (e.g., "Arsenik" as total As) are converted to worst-case CLP compounds (e.g., As₂O₃) using molecular weight conversion factors. PAHs with harmonised CLP entries are mapped directly by CAS number.

2. **Unit conversion**: Lab values in mg/kg TS are converted to % w/w by dividing by 10,000.

3. **HP evaluation**: Each HP criterion is evaluated according to its specific formula:
   - **Individual-limit criteria** (HP7, HP10, HP11, HP13): any single substance ≥ threshold → FA
   - **Summation criteria** (HP4, HP5, HP6, HP8): sum of all substances with relevant H-codes ≥ threshold → FA
   - **HP14 (ecotoxic)**: multiple parallel summation checks with M-factors

4. **Result**: Binary — FA (farligt avfall) if any HP criterion triggers, otherwise Icke FA.

## Difference from NV KM/MKM Classification

| Aspect | NV riktvärden (KM/MKM) | FA (HP-bedömning) |
|--------|----------------------|-------------------|
| Result type | Ordinal (KM < MKM < > MKM) | Binary (FA / Icke FA) |
| Method | Simple threshold comparison | CLP-based HP criteria evaluation |
| Unit | mg/kg TS (direct) | % w/w (converted from mg/kg) |
| Speciation | Not needed (element-based) | Required (worst-case compound) |
| Purpose | Land use suitability | Waste classification for disposal |
| Regulatory basis | NV rapport 5976 | EU 1357/2014 + 2017/997 |

## Below-Detection Handling

- **Individual-limit checks**: below-detection values do not trigger (conservative in favor of non-hazardous, as the actual value could be anywhere from 0 to the detection limit).
- **Summation checks**: below-detection values contribute 0 to the sum (standard practice).
