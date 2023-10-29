use blackbox_log::data::ParserEvent;
use blackbox_log::frame::Frame as _;
use blackbox_log::File;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

static DATA: &[u8] = include_bytes!("../tests/logs/error-recovery.bbl");

fn headers(c: &mut Criterion) {
    c.bench_function("headers", |b| {
        b.iter_batched_ref(|| File::new(DATA), |f| f.parse(0), BatchSize::SmallInput);
    });
}

fn data(c: &mut Criterion) {
    let headers = File::new(DATA).parse(0).unwrap().unwrap();

    c.bench_function("data", |b| {
        b.iter(|| {
            let mut parser = headers.data_parser();
            while let Some(event) = parser.next() {
                match event {
                    ParserEvent::Event(event) => black_box(event),
                    ParserEvent::Main(main) => main.iter().for_each(black_box),
                    ParserEvent::Slow(slow) => slow.iter().for_each(black_box),
                    ParserEvent::Gps(gps) => gps.iter().for_each(black_box),
                }
            }
        });
    });
}

fn black_box<T>(x: T) {
    std::hint::black_box(x);
}

criterion_group!(benches, headers, data);
criterion_main!(benches);
