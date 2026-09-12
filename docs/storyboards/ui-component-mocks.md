# NyxForge UI Component Mocks

> Version 1.0 (April 2026)
> Theme: Cyber-Noir (DarkFi inspired)

---

## 1. Color Palette

| Name | Hex | Usage |
| :--- | :--- | :--- |
| Background | `#0d0f1a` | Main application background |
| Surface | `#1e2030` | Card and sidebar background |
| Primary | `#5e6ad2` | Actions, active borders, progress fill |
| Secondary | `#2a2d3e` | Inactive elements, secondary text |
| Success | `#4ade80` | REDEEMABLE, SETTLED states |
| Warning | `#fbbf24` | DRAFT, Pending Oracle states |
| Danger | `#f87171` | EXPIRED, RECLAIMED states |
| Text Main | `#ffffff` | High emphasis text |
| Text Dim | `#a0a8d0` | Secondary metadata |

---

## 2. Component: Bond Card

The Bond Card is the primary unit of the "Vault" (Bond List).

### 2.1 Visual Mock (ASCII)

```text
┌───────────────────────────────────────────────────────────┐
│ [ ACTIVE ]                                          [ ⋮ ] │
│                                                           │
│ US Homelessness Reduction 2030                            │
│ ───────────────────────────────────────────────────────── │
│ Amount:  1.0 XMR                        Deadline: 2030    │
│ Price:   0.85 XMR (Fair Value)          Remaining: 1,450d │
│                                                           │
│ Goal Progress: 42% (Current: 58,000 / Target: 50,000)     │
│ [▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] │
│                                                           │
│ Oracles: [●] [●] [○] (2/3 Quorum)                         │
└───────────────────────────────────────────────────────────┘
```

### 2.2 Behavior Specs
- **Hover:** Border glows `Primary` (#5e6ad2).
- **Click:** Opens Bond Inspector (Detail View).
- **State Changes:**
    - **REDEEMABLE:** Card flashes `Success` (#4ade80) and a large "REDEEM" button appears over the progress bar.
    - **DRAFT:** "ISSUE" button visible in header.
- **Progress Bar:** Only shown for `quantitative` or `hybrid` bonds.

---

## 3. Component: Wizard Stepper

Used during the `bond create` flow.

### 3.1 Visual Mock (ASCII)

```text
 ( 1 ) ─────── ( 2 ) ─────── ( 3 ) ─────── ( 4 ) ─────── ( 5 )
 Goal          Timing        Collateral    Oracles       Review
 [Active]      [Pending]     [Locked]      [Locked]      [Locked]
```

---

## 4. Component: Oracle Status Row

Detailed view inside the Bond Inspector.

### 4.1 Visual Mock (ASCII)

```text
Oracle: 03a1b2c3... [ ed25519 ]
Role: Quantitative Attestor
Status:
  [✔] Accepted (2026-04-15)
  [✔] s_met Committed (Adaptor Active)
  [○] Attestation Pending (Evaluating HUD-PIT-2030)
```

---

## 5. Component: The sidebar

### 5.1 Visual Mock (ASCII)

```text
┌───────────┐
│ NYX FORGE │
│ ───────── │
│ [■] Vault │  <-- Active state
│ [ ] Hub   │
│ [ ] Wizard│
│           │
│ [ ] Oracle│
│ [ ] Miner │
│           │
│ ───────── │
│ XMR: 4.5  │
│ [■■■■░░░] │
│ 1.2 kH/s  │
└───────────┘
```

---

## 6. Implementation Strategy (Phase 3)

These components will be implemented in Flutter using a CustomPainter for the
grid backgrounds and `Glassmorphism` for the card surfaces. Mockoon will
provide the data stream to test the state transitions.
