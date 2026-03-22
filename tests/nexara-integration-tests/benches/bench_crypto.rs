//! Benchmarks for nexara-crypto operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nexara_crypto::{
    KeyPair, Signer, Blake3Hash, Sha3Hash, BatchVerifier,
    kem::{KemKeyPair, encapsulate, decapsulate},
    mpc::{split_key, reconstruct_key},
};

fn bench_keypair_generation(c: &mut Criterion) {
    c.bench_function("KeyPair::generate", |b| {
        b.iter(|| black_box(KeyPair::generate()))
    });
}

fn bench_signing(c: &mut Criterion) {
    let kp = KeyPair::generate();
    let signer = Signer::new(kp);
    let message = b"benchmark message for signing";

    c.bench_function("Signer::sign", |b| {
        b.iter(|| black_box(signer.sign(black_box(message))))
    });
}

fn bench_verification(c: &mut Criterion) {
    let kp = KeyPair::generate();
    let signer = Signer::new(kp.clone());
    let message = b"benchmark message for verification";
    let sig = signer.sign(message);

    c.bench_function("Signer::verify", |b| {
        b.iter(|| {
            black_box(Signer::verify(
                black_box(&kp.public),
                black_box(message),
                black_box(&sig),
            ))
        })
    });
}

fn bench_batch_verification(c: &mut Criterion) {
    let mut items = Vec::new();
    for _ in 0..10 {
        let kp = KeyPair::generate();
        let signer = Signer::new(kp.clone());
        let msg = b"batch bench message".to_vec();
        let sig = signer.sign(&msg);
        items.push((kp.public.clone(), msg, sig));
    }

    c.bench_function("BatchVerifier::verify_all (10 sigs)", |b| {
        b.iter(|| {
            let mut bv = BatchVerifier::new();
            for (pk, msg, sig) in &items {
                bv.add(pk.clone(), msg.clone(), sig.clone());
            }
            black_box(bv.verify_all())
        })
    });
}

fn bench_blake3_hash(c: &mut Criterion) {
    let data = vec![0xABu8; 1024];

    c.bench_function("Blake3Hash::compute (1KB)", |b| {
        b.iter(|| black_box(Blake3Hash::compute(black_box(&data))))
    });
}

fn bench_blake3_hash_large(c: &mut Criterion) {
    let data = vec![0xCDu8; 1024 * 1024]; // 1 MB

    c.bench_function("Blake3Hash::compute (1MB)", |b| {
        b.iter(|| black_box(Blake3Hash::compute(black_box(&data))))
    });
}

fn bench_sha3_hash(c: &mut Criterion) {
    let data = vec![0xEFu8; 1024];

    c.bench_function("Sha3Hash::compute (1KB)", |b| {
        b.iter(|| black_box(Sha3Hash::compute(black_box(&data))))
    });
}

fn bench_kem_keygen(c: &mut Criterion) {
    c.bench_function("KemKeyPair::generate", |b| {
        b.iter(|| black_box(KemKeyPair::generate()))
    });
}

fn bench_kem_encapsulate(c: &mut Criterion) {
    let kem_kp = KemKeyPair::generate();

    c.bench_function("encapsulate", |b| {
        b.iter(|| black_box(encapsulate(black_box(&kem_kp.public))))
    });
}

fn bench_kem_decapsulate(c: &mut Criterion) {
    let kem_kp = KemKeyPair::generate();
    let (ct, _) = encapsulate(&kem_kp.public);

    c.bench_function("decapsulate", |b| {
        b.iter(|| black_box(decapsulate(black_box(&kem_kp.private), black_box(&ct))))
    });
}

fn bench_mpc_split(c: &mut Criterion) {
    let kp = KeyPair::generate();

    c.bench_function("mpc::split_key (3-of-5)", |b| {
        b.iter(|| black_box(split_key(black_box(&kp.private), 5, 3).unwrap()))
    });
}

fn bench_mpc_reconstruct(c: &mut Criterion) {
    let kp = KeyPair::generate();
    let shares = split_key(&kp.private, 5, 3).unwrap();
    let threshold_shares: Vec<_> = shares.into_iter().take(3).collect();

    c.bench_function("mpc::reconstruct_key (3 shares)", |b| {
        b.iter(|| black_box(reconstruct_key(black_box(&threshold_shares)).unwrap()))
    });
}

criterion_group!(
    benches,
    bench_keypair_generation,
    bench_signing,
    bench_verification,
    bench_batch_verification,
    bench_blake3_hash,
    bench_blake3_hash_large,
    bench_sha3_hash,
    bench_kem_keygen,
    bench_kem_encapsulate,
    bench_kem_decapsulate,
    bench_mpc_split,
    bench_mpc_reconstruct,
);
criterion_main!(benches);
