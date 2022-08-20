use biterator::Biterator;
use blackbox::{encoding, LogVersion};
use criterion::{criterion_group, criterion_main};
use criterion::{BatchSize, Bencher, BenchmarkId, Criterion, Throughput};
use std::iter;

macro_rules! get_bench {
    ($func:expr) => {
        get_bench!(bench, $func)
    };
    ($name:ident, $func:expr) => {
        fn $name(b: &mut Bencher, input: &[u8]) {
            b.iter_batched_ref(|| Biterator::new(input), $func, BatchSize::SmallInput);
        }
    };
}

fn variable(c: &mut Criterion) {
    get_bench!(ubench, encoding::read_uvar);
    get_bench!(ibench, encoding::read_ivar);

    let mut group = c.benchmark_group("variable byte");

    let benches: [(_, &[u8]); 2] = [("min", &[0]), ("max", &[0xFF, 0xFF, 0xFF, 0xFF, 0xF8])];
    for (name, input) in benches {
        group.throughput(Throughput::Bytes(input.len() as u64));

        let id = BenchmarkId::new("unsigned", name);
        group.bench_with_input(id, input, ubench);

        let id = BenchmarkId::new("signed", name);
        group.bench_with_input(id, input, ibench);
    }

    for extra_bytes in 1..5 {
        let mut input: Vec<_> = iter::repeat(0x80).take(extra_bytes).collect();
        input.push(0);
        let input = input.as_slice();
        let name = &format!("{}-byte 0", input.len());

        group.throughput(Throughput::Bytes(input.len() as u64));

        let id = BenchmarkId::new("unsigned", name);
        group.bench_with_input(id, input, ubench);

        let id = BenchmarkId::new("signed", name);
        group.bench_with_input(id, input, ibench);
    }

    group.finish();
}

fn elias_delta(c: &mut Criterion) {
    get_bench!(ubench, encoding::read_u32_elias_delta);
    get_bench!(ibench, encoding::read_i32_elias_delta);

    let mut group = c.benchmark_group("Elias delta");

    let min: &[u8] = &[0x80];
    let max: &[u8] = &[0x04, 0x1F, 0xFF, 0xFF, 0xFF, 0xC0];
    for (input, name) in [(min, "minimum"), (max, "maximum")] {
        group.throughput(Throughput::Bytes(input.len() as u64));

        let id = BenchmarkId::new("unsigned", name);
        group.bench_with_input(id, input, ubench);

        let id = BenchmarkId::new("signed", name);
        group.bench_with_input(id, input, ibench);
    }

    group.finish();
}

fn tagged_16(c: &mut Criterion) {
    use LogVersion::{V1, V2};

    fn get_bench(version: LogVersion) -> impl FnMut(&mut Bencher, &[u8]) {
        move |b, input| {
            b.iter_batched_ref(
                || Biterator::new(input),
                |input| encoding::read_tagged_16(version, input),
                BatchSize::SmallInput,
            )
        }
    }

    fn input(first: u8, zeros: usize) -> Vec<u8> {
        iter::once(first)
            .chain(iter::repeat(0).take(zeros))
            .collect()
    }

    let mut group = c.benchmark_group("tagged 16");

    let benches = [
        (BenchmarkId::from_parameter("zeros"), input(0x00, 0), V1),
        (BenchmarkId::new("v1", "nibbles"), input(0x55, 2), V1),
        (BenchmarkId::new("v2", "nibbles"), input(0x55, 2), V2),
        (BenchmarkId::from_parameter("bytes"), input(0xAA, 4), V1),
        (BenchmarkId::from_parameter("16 bits"), input(0xFF, 8), V1),
    ];
    for (id, input, version) in benches {
        let input = input.as_slice();

        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(id, input, get_bench(version));
    }

    group.finish();
}

criterion_group!(benches, variable, elias_delta, tagged_16);
criterion_main!(benches);
