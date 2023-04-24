#![allow(clippy::missing_panics_doc)]
use criterion::{criterion_group, criterion_main, Criterion};

pub fn run_benchmarks(c: &mut Criterion) {
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain("nightly")
        .manifest_path("../test-apis/comprehensive_api/Cargo.toml")
        .build()
        .unwrap();

    c.bench_function("process JSON", |b| {
        b.iter(|| {
            let _ = public_api::Builder::from_rustdoc_json(&rustdoc_json)
                .sorted(false) // We don't care about sorting time
                .build()
                .unwrap();
        });
    });
}

criterion_group!(benchmarks, run_benchmarks);
criterion_main!(benchmarks);
