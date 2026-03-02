# Bond Lifecycle

## State Machine

```
                     ┌─────────┐
                     │  DRAFT  │  ← issuer defines goal, no collateral yet
                     └────┬────┘
                          │ IssueBond (lock collateral, mint notes)
                          ▼
                     ┌─────────┐
        ┌────────────│  ACTIVE │────────────┐
        │            └────┬────┘            │
        │                 │                 │
        │         oracle attestations       │
        │         accumulate (gossip)       │
        │                 │                 │
        │         quorum reached            │  deadline passed,
        │                 │                 │  goal NOT met
        │                 ▼                 ▼
        │          ┌────────────┐     ┌─────────┐
        │          │ REDEEMABLE │     │ EXPIRED │
        │          └─────┬──────┘     └────┬────┘
        │                │                 │
        │      holders submit BurnProofs   │ issuer reclaims collateral
        │                │                 │
        │                ▼                 │
        │          ┌──────────┐            │
        └─────────►│ SETTLED  │◄───────────┘
                   └──────────┘
```

## Lifecycle Events

### DRAFT → ACTIVE: `IssueBond`

Required:
- `Bond` struct with valid `GoalSpec`, `OracleSpec`, `VerificationCriteria`
- Collateral proof: `total_supply × redemption_value` DRK locked in escrow
- Bond ID must match canonical derivation
- `oracle.quorum > 0` and at least one `oracle_key`

On success:
- Bond series recorded in contract state
- Initial bond notes minted (ZK mint proofs)
- Announcement gossiped to P2P network

### ACTIVE: Trading

Any holder can trade bond notes anonymously via the order book contract.
Each trade requires:
- `TransferProof` for bond notes (seller → buyer)
- Payment `TransferProof` for DRK (buyer → seller)
- Both nullifiers must be unspent

### ACTIVE → REDEEMABLE: Oracle Verification

1. Registered oracle nodes monitor the bond and fetch data.
2. Each oracle posts a signed `OracleAttestation` (goal_met: true/false).
3. Once `quorum` consistent attestations are recorded, any peer can call `FinaliseVerification`.
4. Bond state transitions to REDEEMABLE (goal met) or EXPIRED (not met).
5. A challenge window (`challenge_period_secs`) allows dispute before finalisation.

### REDEEMABLE: Redemption

1. Bond holder generates a `BurnProof`:
   - Proves ownership of a bond note
   - Includes the `quorum_result_hash`
   - Commits to a new DRK payout note
2. Contract verifies the proof, marks nullifier spent, records payout note.
3. Holder can spend the payout note like any other DRK.

### EXPIRED: Collateral Reclaim

If the goal is not met by the deadline:
1. Bond state set to EXPIRED.
2. Issuer submits a `ClaimExpiredCollateral` transaction.
3. Escrowed DRK returned to the issuer's wallet.
4. Bond notes become worthless (no redemption possible).

## Goal Specification Examples

### Environmental

```
GoalMetric {
    data_id:   "noaa.co2.monthly_mean_ppm",
    operator:  LessThan,
    threshold: 350.0,
    aggregation: Some("annual_mean"),
}
deadline: 2045-01-01
```

### Public Health

```
GoalMetric {
    data_id:   "who.malaria.deaths_per_100k",
    operator:  LessThan,
    threshold: 1.0,
    aggregation: Some("global_annual"),
}
deadline: 2035-01-01
```

### Housing

```
GoalMetric {
    data_id:   "us.hud.pit_count.unsheltered",
    operator:  LessThanOrEqual,
    threshold: 50000,
    aggregation: Some("annual_point_in_time"),
}
deadline: 2030-01-01
```

## Pricing Dynamics

Bond price on the secondary market reflects the market's probability estimate
that the goal will be achieved before the deadline:

```
price ≈ redemption_value × P(goal met before deadline)
```

As the deadline approaches and the metric improves (or worsens), the price
adjusts accordingly.  This price signal is itself useful social information.
