# Waste Matrices — Current and Future Support

## Currently Supported

### Jord (Soil)
- **Waste codes:** 17 05 03* (hazardous) / 17 05 04 (non-hazardous)
- **Report format:** Eurofins standard soil analysis
- **Classification:** NV riktvärden (KM/MKM) + FA (HP-bedömning)
- **Parameters:** 11 metals, PAH (L/M/H groups + individual PAH-16), aliphatics, aromatics, BTEX

### Asfalt (Asphalt)
- **Waste codes:** 17 03 01* (hazardous) / 17 03 02 (non-hazardous)
- **Report format:** Eurofins standard asphalt analysis
- **Classification:** PAH-16 total (asfalt preset) + FA (HP-bedömning)
- **Parameters:** Individual PAH-16, PAH-16 sum

## Tier 1 Future — Same Report Format

These matrices use the same Eurofins analysis packages and parameter sets as soil. The HP engine works without modification; only matrix detection and threshold rulesets may need updates.

### Betong (Concrete/Brick)
- **Waste codes:** 17 01 06* (hazardous) / 17 01 07 (non-hazardous)
- **Additional substances needed:** PCB-7 (7 congeners), possibly asbest (qualitative)
- **Implementation:**
  1. Add `Matrix::Betong` variant
  2. Matrix detection in `parsing/header.rs` for "betong", "tegel", "brick"
  3. Add PCB congener names to `parsing/normalize.rs`
  4. Add PCB CLP entries to `clp-substances.json` (PCB-7 are PBT/vPvB substances)
  5. Optionally: betong-specific threshold ruleset

### Sediment
- **Waste codes:** varies by context
- **Additional substances needed:** TBT (tributyltin), possibly additional metals (Sn)
- **Implementation:**
  1. Add `Matrix::Sediment` variant
  2. Matrix detection for "sediment"
  3. Add TBT to normalize aliases and CLP database
  4. HP engine works as-is

### Slam (Sludge)
- **Waste codes:** depends on origin (industrial, municipal)
- **Additional substances needed:** Possibly PFAS, pharmaceutical residues (long-term)
- **Implementation:**
  1. Add `Matrix::Slam` variant
  2. Matrix detection for "slam", "sludge"
  3. Substance additions as needed
  4. HP engine works as-is

## Tier 2 Future — Requires Leaching Value Support

These matrices require evaluation of leaching test results (EN 12457-4) in addition to total concentration, which is a different analytical paradigm.

### Flygaska (Fly Ash)
- **Waste codes:** 10 01 04* (hazardous) / 10 01 15 (non-hazardous)
- **Key difference:** Classification often depends on leaching values (L/S=10)
- **Requirements:**
  1. Leaching value parsing from lab reports
  2. Leaching-based classification criteria (Council Decision 2003/33/EC)
  3. New unit handling (mg/L, µg/L)

### Slagg (Slag)
- **Waste codes:** varies by origin (steel, copper, etc.)
- **Key difference:** Similar to flygaska — leaching values critical
- **Requirements:** Same as flygaska

## Out of Scope

### Grundvatten (Groundwater)
Not waste — regulated under different framework (SGU guidelines, environmental quality standards). Different measurement units (µg/L) and reference values.

### Isolering / Asbest (Insulation / Asbestos)
Qualitative identification (microscopy, PLM/SEM), not quantitative chemical analysis. Different classification approach entirely.

## Matrix-Agnostic HP Engine Design

The HP/CLP engine is **matrix-agnostic by design**. The same engine evaluates any solid waste matrix because:

1. HP criteria are defined by substance concentration (% w/w), not by waste type
2. Speciation assumptions apply to elemental metals regardless of matrix
3. CLP harmonised classifications are substance-specific, not waste-specific

Adding a new solid waste matrix requires only:
1. New `Matrix` enum variant
2. Matrix detection in header parsing
3. Substance aliases for any new analytes
4. CLP database entries for new substances
5. No changes to the HP evaluation engine itself
