use biterator::Biterator;
use blackbox::{encoding, LogVersion};
use criterion::{criterion_group, criterion_main};
use criterion::{BatchSize, Bencher, BenchmarkId, Criterion, Throughput};
use std::iter;

macro_rules! bench {
    ($func:expr) => {
        bench!(bench, $func)
    };
    ($name:ident, $func:expr) => {
        fn $name(b: &mut Bencher, input: &[u8]) {
            b.iter_batched_ref(|| Biterator::new(input), $func, BatchSize::SmallInput);
        }
    };
}

fn uvar(c: &mut Criterion) {
    bench!(encoding::read_uvar);

    let mut group = c.benchmark_group("unsigned variable byte");

    let input: &[u8] = &[0x00];
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input("minimum", input, bench);

    let input: &[u8] = &[0xFF, 0xFF, 0xFF, 0xFF, 0xF8];
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input("maximum", input, bench);

    for extra_bytes in 1..5 {
        let mut input: Vec<_> = iter::repeat(0x80).take(extra_bytes).collect();
        input.push(0);
        let input = input.as_slice();

        group.throughput(Throughput::Bytes(input.len() as u64));

        let id = BenchmarkId::new("multi-byte 0", extra_bytes);
        group.bench_with_input(id, input, bench);
    }

    group.finish();
}

fn ivar(c: &mut Criterion) {
    bench!(encoding::read_ivar);

    let mut group = c.benchmark_group("signed variable byte");

    let input: &[u8] = &[0x00];
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input("minimum", input, bench);

    let input: &[u8] = &[0xFF, 0xFF, 0xFF, 0xFF, 0xF8];
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_with_input("positive maximum", input, bench);

    for extra_bytes in 1..5 {
        let mut input: Vec<_> = iter::repeat(0x80).take(extra_bytes).collect();
        input.push(0);
        let input = input.as_slice();

        group.throughput(Throughput::Bytes(input.len() as u64));

        let id = BenchmarkId::new("multi-byte 0", extra_bytes);
        group.bench_with_input(id, input, bench);
    }

    group.finish();
}

fn u32_elias_delta(c: &mut Criterion) {
    bench!(encoding::read_u32_elias_delta);

    let mut group = c.benchmark_group("unsigned Elias delta");

    let min: &[u8] = &[0x80];
    let max: &[u8] = &[0x04, 0x1F, 0xFF, 0xFF, 0xFF, 0xC0];
    for (input, name) in [(min, "minimum"), (max, "maximum")] {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(name, input, bench);
    }

    group.finish();
}

fn i32_elias_delta(c: &mut Criterion) {
    bench!(encoding::read_u32_elias_delta);

    let mut group = c.benchmark_group("signed Elias delta");

    let min: &[u8] = &[0x80];
    let max: &[u8] = &[0x04, 0x1F, 0xFF, 0xFF, 0xFF, 0x80];
    for (input, name) in [(min, "minimum"), (max, "positive maximum")] {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(name, input, bench);
    }

    group.finish();
}

fn tagged_16(c: &mut Criterion) {
    bench!(bench_v1, |data| encoding::read_tagged_16(
        LogVersion::V1,
        data
    ));
    bench!(bench_v2, |data| encoding::read_tagged_16(
        LogVersion::V2,
        data
    ));

    fn input(first: u8, zeros: usize) -> Vec<u8> {
        iter::once(first)
            .chain(iter::repeat(0).take(zeros))
            .collect()
    }

    let mut group = c.benchmark_group("tagged 16");

    let benches: [(&str, Vec<u8>); 4] = [
        ("zeros", input(0x00, 0)),
        ("nibbles", input(0x55, 2)),
        ("bytes", input(0xAA, 4)),
        ("16 bits", input(0xFF, 8)),
    ];
    for (name, input) in benches.iter() {
        let input = input.as_slice();

        group.throughput(Throughput::Bytes(input.len() as u64));
        let id = BenchmarkId::new("v1", name);
        group.bench_with_input(id, input, bench_v1);

        let id = BenchmarkId::new("v2", name);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(id, input, bench_v2);
    }

    group.finish();
}

criterion_group!(
    benches,
    uvar,
    ivar,
    u32_elias_delta,
    i32_elias_delta,
    tagged_16
);
criterion_main!(benches);
