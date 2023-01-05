use blackbox_log::data::ParseEvent;
use blackbox_log::frame::Frame as _;
use blackbox_log::{DataParser, Headers, Reader};
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
            || reader.clone(),
            |data| {
                let mut parser = DataParser::new(data, &headers);
                while let Some(event) = parser.next() {
                    match event {
                        ParseEvent::Event(event) => black_box(event),
                        ParseEvent::Main(main) => main.iter().for_each(black_box),
                        ParseEvent::Slow(slow) => slow.iter().for_each(black_box),
                        ParseEvent::Gps(gps) => gps.iter().for_each(black_box),
                    }
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn black_box<T>(x: T) {
    std::hint::black_box(x);
}

criterion_group!(benches, headers, data);
criterion_main!(benches);
