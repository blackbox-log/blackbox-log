use blackbox_log::parser::{Headers, Reader};
use blackbox_log::Log;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

static DATA: &[u8] = include_bytes!("../tests/logs/error-recovery.bbl");

fn headers(c: &mut Criterion) {
    c.bench_function("headers", |b| {
        b.iter_batched_ref(|| Reader::new(DATA), Headers::parse, BatchSize::SmallInput);
    });
}

fn data(c: &mut Criterion) {
    let mut reader = Reader::new(DATA);
    let headers = Headers::parse(&mut reader).unwrap();

    c.bench_function("data", |b| {
        b.iter_batched(
            || (reader.clone(), headers.clone()),
            |(data, headers)| Log::parse_with_headers(data, headers),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, headers, data);
criterion_main!(benches);
