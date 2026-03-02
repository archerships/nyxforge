# ZK Circuit Design

NyxForge uses DarkFi's zkVM (Halo2-based) for all privacy-preserving proofs.
This document describes the circuits at a conceptual level.

## Note Model

A bond note is an anonymous commitment:

```
Commitment = PedersenCommit(bond_id ‖ quantity ‖ owner_pubkey; randomness)
```

The commitment is public (recorded on-chain).  The plaintext is encrypted
to `owner_pubkey` using an integrated encryption scheme (IES/ECIES variant).

## Circuits

### MINT

**Purpose:** Prove that a new note commitment is well-formed without revealing
the owner or the randomness.

```
Public inputs:
  C          : note commitment
  bond_id    : bond series
  quantity   : (optionally public, or range-proved)

Private witness:
  owner_pk   : recipient's public key
  r          : blinding randomness
  serial     : fresh serial number

Constraints:
  C == PedersenCommit(bond_id ‖ quantity ‖ owner_pk; r)
  serial has not been used before (checked via nullifier set off-circuit)
```

### TRANSFER

**Purpose:** Prove ownership transfer preserves quantity (no inflation).

```
Public inputs:
  N_old      : nullifier of the spent note
  C_new      : commitment to the new note
  bond_id    : must match both notes

Private witness:
  note_old   : { bond_id, quantity, owner_pk, r, serial }
  owner_sk   : secret key matching owner_pk
  note_new   : { bond_id, quantity, new_owner_pk, r', serial' }

Constraints:
  C_old == PedersenCommit(note_old)           // old note is valid
  N_old == PRF(owner_sk, note_old.serial)     // nullifier derivation
  PubKey(owner_sk) == note_old.owner_pk       // caller owns the note
  note_old.quantity == note_new.quantity      // conservation
  note_old.bond_id  == note_new.bond_id       // same series
  C_new == PedersenCommit(note_new)           // new note is valid
```

### BURN (Redemption)

**Purpose:** Prove ownership of a redeemable bond note and authorise payout.

```
Public inputs:
  N              : nullifier of the burned bond note
  bond_id        : bond series
  Q_hash         : blake3(QuorumResult) — goal verification result
  C_payout       : commitment to the DRK payout note
  payout_amount  : quantity × redemption_value (public)

Private witness:
  bond_note      : { bond_id, quantity, redemption_value, owner_pk, r, serial }
  owner_sk       : secret key
  payout_r       : randomness for payout note

Constraints:
  N == PRF(owner_sk, bond_note.serial)
  PubKey(owner_sk) == bond_note.owner_pk
  payout_amount == bond_note.quantity × bond_note.redemption_value
  C_payout == PedersenCommit(payout_amount ‖ owner_pk; payout_r)
  Q_hash committed to by the contract (checked off-circuit against state)
```

## Trusted Setup

DarkFi uses an updateable universal reference string (URS) for Halo2.
NyxForge does not require a circuit-specific trusted setup, as Halo2's
proof system is transparent (no toxic waste).

## Threat Model

| Threat                         | Mitigation                              |
|--------------------------------|-----------------------------------------|
| Double spend                   | Nullifier set (on-chain, public)        |
| Note forgery                   | MINT proof; issuer controls supply      |
| Inflation via transfer         | TRANSFER conservation constraint        |
| False oracle attestation       | Slash collateral; quorum required       |
| Sybil oracle attack            | Stake requirement per oracle            |
| Linkability of trades          | ZK transfer; fresh randomness per note  |
| Wallet data leakage            | Keys never leave device; WASM sandbox   |
| Redemption without goal met    | BURN proof includes quorum_result_hash  |
