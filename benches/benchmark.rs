use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
    Throughput,
};

const DATA: &[u8; 1024 * 1024] = include_bytes!("seed.bin");

fn bench_compare_hex(c: &mut Criterion) {
    let mut group = c.benchmark_group("encode/decode 1M");
    group.throughput(Throughput::Bytes(DATA.len() as u64));

    group.bench_function(BenchmarkId::new("encode", "hex"), |b| {
        b.iter(|| hex::encode(black_box(&DATA)))
    });

    group.bench_function(BenchmarkId::new("encode", "muhex"), |b| {
        b.iter(|| muhex::encode(black_box(&DATA)))
    });

    let data = muhex::encode(DATA);

    group.bench_function(BenchmarkId::new("decode", "hex"), |b| {
        b.iter(|| hex::decode(black_box(&data)).unwrap())
    });

    group.bench_function(BenchmarkId::new("decode", "muhex"), |b| {
        b.iter(|| muhex::decode(black_box(&data)).unwrap())
    });
}

criterion_group!(benches, bench_compare_hex);
criterion_main!(benches);
