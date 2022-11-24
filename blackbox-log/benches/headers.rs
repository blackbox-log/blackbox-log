use blackbox_log::parser::headers::parse_headers;
use criterion::{criterion_group, criterion_main, Criterion};

static DATA: &[u8] = include_bytes!("../tests/logs/error-recovery.bbl");

fn parse(c: &mut Criterion) {
    c.bench_function("parse", |b| {
        b.iter(|| parse_headers(DATA));
    });
}

criterion_group!(benches, parse);
criterion_main!(benches);
