// benches/benchmark.rs
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn expensive_function() {}

fn bench_fn(c: &mut Criterion) {
    c.bench_function("expensive_func", |b| {
        b.iter(|| black_box(expensive_function()))
    });
}

// criterion_group!(
//     name = benches;
//     config = Criterion::default().sample_size(10);
//     targets = bench_fn
// );
criterion_group!(benches, bench_fn);
criterion_main!(benches);
