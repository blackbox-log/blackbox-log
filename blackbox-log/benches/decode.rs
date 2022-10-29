use std::fmt::Display;
use std::iter;

use blackbox_log::parser::{decode, Reader};
use criterion::measurement::WallTime;
use criterion::{
    criterion_group, criterion_main, BatchSize, Bencher, BenchmarkGroup, BenchmarkId, Criterion,
    Throughput,
};

fn get_optimized_input(input: &[u8]) -> Vec<u8> {
    let mut input = input.to_owned();
    input.extend(iter::from_fn(|| Some(0)).take(8));
    input
}

fn get_bytes_reader<'a>(input: &'a [u8]) -> impl Fn() -> Reader<'a> {
    || Reader::new(input)
}

macro_rules! get_bench {
    ($func:expr) => {
        get_bench!(bench, $func)
    };
    ($name:ident, $func:expr) => {
        fn $name(b: &mut Bencher, input: &[u8]) {
            let input = &get_optimized_input(input);
            b.iter_batched_ref(get_bytes_reader(input), $func, BatchSize::SmallInput);
        }
    };
}

fn run_bench_pair<P: Display>(
    group: &mut BenchmarkGroup<WallTime>,
    input: &[u8],
    parameter: P,
    ubench: impl FnMut(&mut Bencher, &[u8]),
    ibench: impl FnMut(&mut Bencher, &[u8]),
) {
    group.throughput(Throughput::Bytes(input.len() as u64));

    let id = BenchmarkId::new("unsigned", &parameter);
    group.bench_with_input(id, input, ubench);

    let id = BenchmarkId::new("signed", &parameter);
    group.bench_with_input(id, input, ibench);
}

fn variable(c: &mut Criterion) {
    get_bench!(ubench, decode::variable);
    get_bench!(ibench, decode::variable_signed);

    let mut group = c.benchmark_group("variable byte");

    let benches: [(_, &[u8]); 2] = [("min", &[0]), ("max", &[0xFF, 0xFF, 0xFF, 0xFF, 0xF8])];
    for (name, input) in benches {
        run_bench_pair(&mut group, input, name, ubench, ibench);
    }

    for extra_bytes in 1..5 {
        let mut input: Vec<_> = iter::repeat(0x80).take(extra_bytes).collect();
        input.push(0);
        let input = input.as_slice();
        let name = &format!("{}-byte 0", input.len());

        run_bench_pair(&mut group, input, name, ubench, ibench);
    }

    group.finish();
}

fn negative_14_bit(c: &mut Criterion) {
    get_bench!(decode::negative_14_bit);

    let mut group = c.benchmark_group("negative 14 bit");

    let benches: [(_, &[u8]); 2] = [("min", &[0]), ("max", &[0x80, 0x40])];
    for (name, input) in benches {
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(name, input, bench);
    }

    group.finish();
}

fn tagged_zeros(first: u8, zeros: usize) -> Vec<u8> {
    iter::once(first)
        .chain(iter::repeat(0).take(zeros))
        .collect()
}

fn tagged_16(c: &mut Criterion) {
    get_bench!(decode::tagged_16);

    let mut group = c.benchmark_group("tagged 16");

    let benches = [
        ("4x00 bits", tagged_zeros(0x00, 0)),
        ("4x04 bits", tagged_zeros(0x55, 2)),
        ("4x08 bits", tagged_zeros(0xAA, 4)),
        ("4x16 bits", tagged_zeros(0xFF, 8)),
    ];
    for (name, input) in benches {
        let input = input.as_slice();

        group.throughput(Throughput::Bytes(input.len() as u64));

        let id = BenchmarkId::from_parameter(name);
        group.bench_with_input(id, input, bench);
    }

    group.finish();
}

fn tagged_32(c: &mut Criterion) {
    get_bench!(decode::tagged_32);

    let mut group = c.benchmark_group("tagged 32");

    let benches = [
        ("3x02 bits", tagged_zeros(0x00, 0)),
        ("3x04 bits", tagged_zeros(0x40, 1)),
        ("3x06 bits", tagged_zeros(0x80, 2)),
        ("3x08 bits", tagged_zeros(0xC0, 3)),
        ("3x16 bits", tagged_zeros(0xD7, 6)),
        ("3x24 bits", tagged_zeros(0xEA, 9)),
        ("3x32 bits", tagged_zeros(0xFF, 12)),
    ];
    for (id, input) in benches {
        let input = input.as_slice();

        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(id, input, bench);
    }

    group.finish();
}

criterion_group!(benches, variable, negative_14_bit, tagged_16, tagged_32);
criterion_main!(benches);
