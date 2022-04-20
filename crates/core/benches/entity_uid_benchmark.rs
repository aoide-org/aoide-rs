use aoide_core::entity::EntityUid;

use criterion::{criterion_group, criterion_main, Criterion};
use uuid::Uuid;

fn random_uuid(n: u64) {
    for _ in 0..n {
        let uuid = Uuid::new_v4();
        assert_ne!(Uuid::nil(), uuid);
        let encoded = uuid.as_simple().to_string();
        assert!(!encoded.is_empty());
    }
}

fn random_entity_uid(n: u64) {
    for _ in 0..n {
        let entity_uid = EntityUid::random();
        let encoded = entity_uid.encode_to_string();
        assert!(!encoded.is_empty());
    }
}

fn generate_entity_uid(n: u64) {
    for _ in 0..n {
        let entity_uid = EntityUid::random();
        let encoded = entity_uid.encode_to_string();
        assert!(!encoded.is_empty());
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("EntityUid vs Uuid");

    let n = 10_000;

    group.bench_function("random_uuid", |b| b.iter(|| random_uuid(n)));
    group.bench_function("random_entity_uid", |b| b.iter(|| random_entity_uid(n)));
    group.bench_function("generate_entity_uid", |b| b.iter(|| generate_entity_uid(n)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
