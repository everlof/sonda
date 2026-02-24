# HP Criteria Implementation

## Overview

Sonda evaluates 9 of the 15 HP criteria defined in EU Regulation 1357/2014 (HP4–HP13) and Commission Regulation 2017/997 (HP14). The remaining criteria (HP1–HP3, HP9, HP12, HP15) are not applicable to chemical analysis of contaminated soil and asphalt.

All concentrations are in **% w/w** (weight/weight percentage). Lab values in mg/kg TS are converted by dividing by 10,000.

## Evaluation Types

### Individual-Limit Criteria

A single substance exceeding the threshold triggers the criterion. Each substance is checked independently.

### Summation Criteria

All substances sharing a given H-code have their concentrations summed. The sum is compared against the threshold.

## HP7 — Carcinogenic

**Type:** Individual limit
**Regulation:** 1357/2014, Annex, point 7

| H-code | Category | Threshold |
|--------|----------|-----------|
| H350 (includes H350i) | Carc. 1A, 1B | ≥ 0.1% |
| H351 | Carc. 2 | ≥ 1.0% |

**Example:** Arsenik 1200 mg/kg → As₂O₃: 1200 × 1.32 / 10000 = 0.1584% ≥ 0.1% → **triggers HP7**.

## HP11 — Mutagenic

**Type:** Individual limit
**Regulation:** 1357/2014, Annex, point 11

| H-code | Category | Threshold |
|--------|----------|-----------|
| H340 | Muta. 1A, 1B | ≥ 0.1% |
| H341 | Muta. 2 | ≥ 1.0% |

## HP10 — Toxic for Reproduction

**Type:** Individual limit
**Regulation:** 1357/2014, Annex, point 10

| H-code | Category | Threshold |
|--------|----------|-----------|
| H360 (all variants: H360FD, H360D, H360F) | Repr. 1A, 1B | ≥ 0.3% |
| H361 (all variants) | Repr. 2 | ≥ 0.3% |

**SCL override:** Lead (Pb, CAS 7439-92-1) has an SCL of 0.03% for Repr. 1A, replacing the GCL of 0.3%.

**Example:** Bly 300 mg/kg → Pb: 300 × 1.0 / 10000 = 0.03% ≥ 0.03% (SCL) → **triggers HP10**.

## HP5 — STOT (Specific Target Organ Toxicity)

**Type:** Mixed (individual + summation)
**Regulation:** 1357/2014, Annex, point 5

Individual limits:

| H-code | Threshold |
|--------|-----------|
| H370 (STOT SE 1) | ≥ 1.0% |
| H371 (STOT SE 2) | ≥ 10.0% |

Summation limits:

| H-code | Threshold |
|--------|-----------|
| H372 (STOT RE 1) | Σ ≥ 1.0% |
| H373 (STOT RE 2) | Σ ≥ 10.0% |

**Relevant substances with H372:** CrO₃, HgCl₂, CdO, V₂O₅.

## HP6 — Acute Toxicity

**Type:** Summation (per H-code)
**Regulation:** 1357/2014, Annex, point 6

| Route | H-code | Category | Threshold |
|-------|--------|----------|-----------|
| Oral | H300 | Acute Tox. 1, 2 | Σ ≥ 0.1% |
| Oral | H301 | Acute Tox. 3 | Σ ≥ 5.0% |
| Oral | H302 | Acute Tox. 4 | Σ ≥ 25.0% |
| Dermal | H310 | Acute Tox. 1, 2 | Σ ≥ 0.1% |
| Dermal | H311 | Acute Tox. 3 | Σ ≥ 5.0% |
| Dermal | H312 | Acute Tox. 4 | Σ ≥ 25.0% |
| Inhalation | H330 | Acute Tox. 1, 2 | Σ ≥ 0.1% |
| Inhalation | H331 | Acute Tox. 3 | Σ ≥ 5.0% |
| Inhalation | H332 | Acute Tox. 4 | Σ ≥ 25.0% |

Each H-code sum is evaluated independently. Any single sum exceeding its threshold triggers HP6.

## HP4 — Irritant

**Type:** Summation
**Regulation:** 1357/2014, Annex, point 4

| H-code | Threshold |
|--------|-----------|
| H315 (Skin Irrit. 2) | Σ ≥ 20.0% |
| H319 (Eye Irrit. 2A) | Σ ≥ 20.0% |

**Note:** With typical soil contamination levels, HP4 is extremely unlikely to trigger.

## HP8 — Corrosive

**Type:** Summation
**Regulation:** 1357/2014, Annex, point 8

| H-code | Threshold |
|--------|-----------|
| H314 (Skin Corr.) | Σ ≥ 5.0% |

**Relevant substances:** CrO₃ (H314), HgCl₂ (H314).

## HP13 — Sensitising

**Type:** Individual limit
**Regulation:** 1357/2014, Annex, point 13

| H-code | Threshold |
|--------|-----------|
| H317 (Skin Sens. 1) | ≥ 10.0% |
| H334 (Resp. Sens. 1) | ≥ 10.0% |

**Relevant substances with H317:** NiSO₄, BaP, CoCl₂.
**Relevant substances with H334:** CrO₃, CoCl₂.

**Note:** At the concentrations typically found in soil, HP13 rarely triggers.

## HP14 — Ecotoxic

**Type:** Multiple parallel summation checks with M-factors
**Regulation:** Commission Regulation (EU) 2017/997

HP14 has 4 parallel evaluation checks. Any one triggering classifies the waste as HP14-positive:

### Check 1: Acute aquatic toxicity
```
Σ(c_i × M_acute) for all H400 substances ≥ 25%
```

### Check 2: Chronic aquatic toxicity (H410)
```
100 × Σ(c_i × M_chronic) for all H410 substances ≥ 25%
```

### Check 3: Combined chronic (H410 + H411)
```
10 × Σ(c_i × M_chronic) for H410 + Σ(c_i) for H411 ≥ 2.5%
```
(No H411 substances in current database.)

### Check 4: All aquatic categories
```
Σ weighted concentrations for H410+H411+H412+H413 ≥ 25%
```
(Only H410 substances in current database, so this reduces to check 2.)

**M-factor defaults:** When no M-factor is specified in Annex VI, M = 1 is used.

**Key M-factors:** Cu₂O has M(acute) = 100, making copper the primary driver for HP14 in soil. ZnO has M(acute) = 10.

**Example:** Koppar 5000 mg/kg → Cu₂O: 5000 × 1.13 / 10000 = 0.565%. Check 1: 0.565 × 100 = 56.5% ≥ 25% → **triggers HP14**.

## Below-Detection Handling

- **Individual-limit criteria:** Below-detection values are excluded from evaluation (they cannot exceed the threshold since the actual value is unknown but ≤ the detection limit).
- **Summation criteria:** Below-detection values contribute 0 to the sum. This is conservative in favor of non-hazardous classification, consistent with standard waste classification practice.

## Test Cases

### Clean soil (Icke FA)
```
Arsenik: 5 mg/kg → As₂O₃: 0.00066% (HP7 threshold: 0.1%) — not triggered
Bly: 20 mg/kg → Pb: 0.002% (HP10 SCL: 0.03%) — not triggered
Koppar: 30 mg/kg → Cu₂O: 0.00339% (HP14 check1: 0.339%) — not triggered
```

### Contaminated soil (FA via HP7)
```
Arsenik: 1200 mg/kg → As₂O₃: 0.1584% >= 0.1% → HP7 triggered → FA
```

### Contaminated soil (FA via HP14)
```
Koppar: 5000 mg/kg → Cu₂O: 0.565%, ×M(100) = 56.5% >= 25% → HP14 triggered → FA
```
