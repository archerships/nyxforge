use criterion::{criterion_group, criterion_main, Criterion};
use nyxforge_core::types::{Amount, PublicKey};
use nyxforge_zk::{
    burn::{BurnProof, BurnWitness},
    mint::{MintProof, MintWitness},
    note::BondNote,
    transfer::{TransferProof, TransferWitness},
};
use nyxforge_core::types::Digest;

fn bench_mint(c: &mut Criterion) {
    let witness = MintWitness {
        bond_id:          Digest::from_bytes([0x01u8; 32]),
        quantity:         10,
        redemption_value: Amount(1_000_000),
        recipient:        PublicKey([0xBBu8; 32]),
        randomness:       [0x42u8; 32],
        serial:           [0x55u8; 32],
    };

    let mut g = c.benchmark_group("mint");
    g.sample_size(10);
    g.bench_function("prove", |b| {
        b.iter(|| MintProof::prove(&witness).unwrap())
    });
    let proof = MintProof::prove(&witness).unwrap();
    g.bench_function("verify", |b| {
        b.iter(|| proof.verify().unwrap())
    });
    g.finish();
}

fn bench_transfer(c: &mut Criterion) {
    let witness = TransferWitness {
        old_note: BondNote {
            bond_id:          Digest::from_bytes([0x01u8; 32]),
            quantity:         10,
            redemption_value: Amount(1_000_000),
            owner:            PublicKey([0xBBu8; 32]),
            randomness:       [0x42u8; 32],
            serial:           [0x55u8; 32],
        },
        owner_secret:   [0xAAu8; 32],
        recipient:      PublicKey([0xCCu8; 32]),
        new_randomness: [0x77u8; 32],
        new_serial:     [0x88u8; 32],
    };

    let mut g = c.benchmark_group("transfer");
    g.sample_size(10);
    g.bench_function("prove", |b| {
        b.iter(|| TransferProof::prove(&witness).unwrap())
    });
    let proof = TransferProof::prove(&witness).unwrap();
    g.bench_function("verify", |b| {
        b.iter(|| proof.verify().unwrap())
    });
    g.finish();
}

fn bench_burn(c: &mut Criterion) {
    let witness = BurnWitness {
        bond_note: BondNote {
            bond_id:          Digest::from_bytes([0x01u8; 32]),
            quantity:         10,
            redemption_value: Amount(1_000_000),
            owner:            PublicKey([0xBBu8; 32]),
            randomness:       [0x42u8; 32],
            serial:           [0x55u8; 32],
        },
        owner_secret:      [0xAAu8; 32],
        quorum_result_hash: Digest::from_bytes([0xDDu8; 32]),
        payout_randomness:  [0x33u8; 32],
    };

    let mut g = c.benchmark_group("burn");
    g.sample_size(10);
    g.bench_function("prove", |b| {
        b.iter(|| BurnProof::prove(&witness).unwrap())
    });
    let proof = BurnProof::prove(&witness).unwrap();
    g.bench_function("verify", |b| {
        b.iter(|| proof.verify().unwrap())
    });
    g.finish();
}

criterion_group!(benches, bench_mint, bench_transfer, bench_burn);
criterion_main!(benches);
