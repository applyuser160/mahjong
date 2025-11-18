use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_sum(c: &mut Criterion) {
    c.bench_function("sum", |b| b.iter(|| sum(black_box(2), black_box(2))));
}

criterion_group!(benches, bench_sum);
criterion_main!(benches);
