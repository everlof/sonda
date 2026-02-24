# CLP Substance Database

## Data Source

All CLP harmonised classifications are sourced from **ECHA Annex VI, Table 3.1** of the CLP Regulation (EC) No 1272/2008. This table contains the official EU harmonised classification and labelling for hazardous substances.

The database is embedded at `rules/clp-substances.json`.

## Included Substances

The database covers substances relevant to contaminated soil and asphalt waste classification (~25 entries):

### Metals (11 entries)

Metals in lab reports are reported as total elemental concentration (e.g., "Arsenik 25 mg/kg"). CLP classification applies to specific compounds, not elements. We use **worst-case speciation** per EU Technical Guidance (Commission notice 2018/C 124/01).

| Lab substance | CLP compound | CAS | Rationale for worst-case |
|--------------|-------------|-----|--------------------------|
| Arsenik | As₂O₃ | 1327-53-3 | Most toxic common arsenic compound, Carc. 1A |
| Kadmium | CdO | 1306-19-0 | Carc. 1B, STOT RE 1 |
| Krom (total) | CrO₃ | 1333-82-0 | Assumes worst-case Cr(VI), Carc. 1A, Muta. 1B |
| Koppar | Cu₂O | 1317-39-1 | High aquatic toxicity M-factors |
| Bly | Pb (group entry) | 7439-92-1 | Lead group entry in Annex VI |
| Kvicksilver | HgCl₂ | 7487-94-7 | Common inorganic mercury compound |
| Nickel | NiSO₄ | 7786-81-4 | Soluble nickel salt, worst-case for Carc. 1A |
| Zink | ZnO | 1314-13-2 | Common zinc compound, aquatic toxicity |
| Barium | BaCl₂ | 10361-37-2 | Soluble barium salt, acute toxicity |
| Kobolt | CoCl₂ | 7646-79-9 | Soluble cobalt salt, Carc. 1B |
| Vanadin | V₂O₅ | 1314-62-1 | Vanadium pentoxide, Carc. 2 |

### PAHs with Harmonised Classification (8 entries)

PAHs are reported as individual compounds in lab reports and have direct CAS number mappings (no speciation conversion needed):

| Substance | CAS | Key classifications |
|-----------|-----|-------------------|
| Naftalen | 91-20-3 | Carc. 2 (H351), Aquatic Acute/Chronic 1 |
| Benso(a)antracen | 56-55-3 | Carc. 1B (H350), Aquatic 1 |
| Krysen | 218-01-9 | Carc. 1B (H350), Muta. 2 (H341), Aquatic 1 |
| Benso(b)fluoranten | 205-99-2 | Carc. 1B (H350), Aquatic 1 |
| Benso(k)fluoranten | 207-08-9 | Carc. 1B (H350), Aquatic 1 |
| Benso(a)pyren | 50-32-8 | Carc. 1B (H350), Muta. 1B (H340), Repr. 1B (H360FD) |
| Dibenso(a,h)antracen | 53-70-3 | Carc. 1B (H350), Aquatic 1 |
| Antracen | 120-12-7 | Skin Irrit. 2 (H315), Aquatic 1 |

### PAHs Without Harmonised Classification (omitted)

The following PAH-16 compounds have **no harmonised CLP entry** and are therefore excluded from HP evaluation: acenaftylen, acenaften, fluoren, fenantren, fluoranten, pyren, benso(ghi)perylen, indeno(1,2,3-cd)pyren.

## Speciation Methodology

### Regulatory Basis

The speciation approach follows the **worst-case assumption** principle from EU Technical Guidance (Commission notice 2018/C 124/01, section 3.2):

> "Where a waste contains a substance in a form which could not be identified [...] the worst-case scenario should be applied."

For metals reported as total elemental concentration, we assume the most hazardous compound form that could plausibly be present in contaminated soil.

### Conversion Factor Derivation

The conversion factor transforms elemental mass to compound mass:

```
factor = MW(compound) / (n × MW(element))
```

where n is the number of element atoms per compound molecule.

| Metal | Compound | Factor | Calculation |
|-------|----------|--------|-------------|
| As | As₂O₃ | 1.32 | 197.84 / (2 × 74.92) |
| Cd | CdO | 1.14 | 128.41 / 112.41 |
| Cr | CrO₃ | 1.92 | 99.99 / 52.00 |
| Cu | Cu₂O | 1.13 | 143.09 / (2 × 63.55) |
| Pb | Pb | 1.00 | Group entry, no conversion |
| Hg | HgCl₂ | 1.35 | 271.52 / 200.59 |
| Ni | NiSO₄ | 2.64 | 154.75 / 58.69 |
| Zn | ZnO | 1.24 | 81.38 / 65.38 |
| Ba | BaCl₂ | 1.52 | 208.23 / 137.33 |
| Co | CoCl₂ | 2.20 | 129.84 / 58.93 |
| V | V₂O₅ | 1.78 | 181.88 / (2 × 50.94) |

## M-Factors for Aquatic Toxicity

M-factors increase the effective concentration for HP14 (ecotoxic) evaluation. Only substances with harmonised M-factors in Annex VI are assigned non-default values:

| Substance | M-factor (acute) | M-factor (chronic) |
|-----------|-----------------|-------------------|
| Cu₂O | 100 | 1 |
| ZnO | 10 | 1 |
| All others | 1 (default) | 1 (default) |

## Specific Concentration Limits (SCLs)

SCLs override the generic concentration limits (GCLs) for specific substance-hazard combinations:

| Substance | SCL | Overrides |
|-----------|-----|-----------|
| Pb (7439-92-1) | Repr. 1A: 0.03% | GCL for H360 of 0.3% |

## Known Limitations

- **No petroleum hydrocarbon fractions**: Aliphatic and aromatic fractions (C5–C35) are UVCB substances with complex classification — not included in current HP evaluation.
- **No PCB-7**: Required for betong (concrete) waste classification — planned for future.
- **No leaching values**: Certain waste types (flygaska, slagg) require leaching value evaluation — out of current scope.
