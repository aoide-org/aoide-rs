use aoide_core::prelude::*;

use criterion::{criterion_group, criterion_main, Criterion, ParameterizedBenchmark};
use uuid::Uuid;

fn random_uuid(n: u64) {
    for _ in 0..n {
        let uuid = Uuid::new_v4();
        assert_ne!(Uuid::nil(), uuid);
        let encoded = uuid.to_simple_ref().to_string();
        assert!(!encoded.is_empty());
    }
}

fn random_entity_uid(n: u64) {
    for _ in 0..n {
        let entity_uid = EntityUid::random();
        assert!(entity_uid.is_valid());
        let encoded = entity_uid.encode_to_string();
        assert!(!encoded.is_empty());
    }
}

fn generate_entity_uid(n: u64) {
    for _ in 0..n {
        let entity_uid = EntityUidGenerator::generate_uid();
        assert!(entity_uid.is_valid());
        let encoded = entity_uid.encode_to_string();
        assert!(!encoded.is_empty());
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench(
        "EntityUid vs Uuid",
        ParameterizedBenchmark::new(
            "random_uuid",
            |b, i| b.iter(|| random_uuid(*i)),
            vec![10_000],
        )
        .with_function("random_entity_uid", |b, i| b.iter(|| random_entity_uid(*i)))
        .with_function("generate_entity_uid", |b, i| {
            b.iter(|| generate_entity_uid(*i))
        }),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
