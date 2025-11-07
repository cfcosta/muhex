use std::time::Duration;

use criterion::{
    BenchmarkId,
    Criterion,
    Throughput,
    black_box,
    criterion_group,
    criterion_main,
};

const DATA: &[u8; 1024 * 1024] = include_bytes!("seed.bin");

fn bench_compare_hex(c: &mut Criterion) {
    let mut group = c.benchmark_group("encdec");
    group.throughput(Throughput::Bytes(DATA.len() as u64));
    group.measurement_time(Duration::from_secs(30));

    group.bench_function(BenchmarkId::new("encode", "hex"), |b| {
        b.iter(|| hex::encode(black_box(&DATA)))
    });

    group.bench_function(BenchmarkId::new("encode", "muhex"), |b| {
        b.iter(|| muhex::encode(black_box(&DATA)))
    });

    group.bench_function(BenchmarkId::new("encode", "faster-hex"), |b| {
        b.iter(|| faster_hex::hex_string(black_box(DATA.as_slice())))
    });

    let data = muhex::encode(DATA);

    group.bench_function(BenchmarkId::new("decode", "hex"), |b| {
        b.iter(|| hex::decode(black_box(&data)).unwrap())
    });

    group.bench_function(BenchmarkId::new("decode", "muhex"), |b| {
        b.iter(|| muhex::decode(black_box(&data)).unwrap())
    });

    group.bench_function(BenchmarkId::new("decode", "faster-hex"), |b| {
        b.iter(|| {
            let mut output = Vec::with_capacity(data.len() / 2);
            faster_hex::hex_decode(
                black_box(data.as_bytes()),
                black_box(&mut output),
            )
            .unwrap();
            black_box(output);
        })
    });

    group.finish();
}

#[cfg(feature = "serde")]
fn bench_serde(c: &mut Criterion) {
    let test_data = DATA.to_vec();

    let mut group = c.benchmark_group("serde");
    group.throughput(Throughput::Bytes(DATA.len() as u64));
    group.measurement_time(Duration::from_secs(30));

    group.bench_function(BenchmarkId::new("serialize", "hex"), |b| {
        b.iter(|| {
            hex::serde::serialize(
                black_box(&test_data),
                serde_json::value::Serializer,
            )
            .unwrap()
        })
    });

    group.bench_function(BenchmarkId::new("serialize", "muhex"), |b| {
        b.iter(|| {
            muhex::serde::serialize(
                black_box(&test_data),
                serde_json::value::Serializer,
            )
            .unwrap()
        })
    });

    let serialized =
        muhex::serialize(&test_data, serde_json::value::Serializer).unwrap();

    group.bench_function(BenchmarkId::new("deserialize", "hex"), |b| {
        b.iter(|| {
            hex::serde::deserialize::<_, Vec<u8>>(black_box(&serialized))
                .unwrap()
        })
    });

    group.bench_function(BenchmarkId::new("deserialize", "muhex"), |b| {
        b.iter(|| {
            muhex::serde::deserialize::<_, Vec<u8>>(black_box(&serialized))
                .unwrap()
        })
    });

    group.finish();
}

#[cfg(not(feature = "serde"))]
fn bench_serde(_c: &mut Criterion) {}

criterion_group!(benches, bench_compare_hex, bench_serde);
criterion_main!(benches);
