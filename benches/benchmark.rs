use std::time::Duration;

use criterion::{
    BenchmarkId,
    Criterion,
    Throughput,
    black_box,
    criterion_group,
    criterion_main,
};

const DATA_1MB: &[u8; 1024 * 1024] = include_bytes!("seed.bin");

fn bench_compare_hex(c: &mut Criterion) {
    let mut group = c.benchmark_group("encdec");
    group.measurement_time(Duration::from_secs(5));

    // Test different sizes to hit different code paths
    let test_cases = [
        ("1MB_aligned", &DATA_1MB[..]), // 1MB - fully aligned
        ("1KB_aligned", &DATA_1MB[..1024]), // 1KB - aligned to 32
        ("96B_aligned", &DATA_1MB[..96]), // 96B - aligned to 32
        ("64B_aligned", &DATA_1MB[..64]), // 64B - exactly one 64-byte chunk
        ("63B_unaligned", &DATA_1MB[..63]), // 63B - tests 32-byte path + remainder
        ("33B_unaligned", &DATA_1MB[..33]), // 33B - tests 32-byte + 1 byte remainder
        ("31B_remainder", &DATA_1MB[..31]), // 31B - no 32-byte chunks, all remainder
        ("17B_small", &DATA_1MB[..17]), // 17B - tests 16-byte SIMD remainder
        ("15B_small", &DATA_1MB[..15]), // 15B - tests smaller SIMD remainder
        ("7B_tiny", &DATA_1MB[..7]),    // 7B - tests LUT-only path
        ("3B_tiny", &DATA_1MB[..3]),    // 3B - minimal LUT path
        ("1B_minimal", &DATA_1MB[..1]), // 1B - edge case
    ];

    for (name, data) in test_cases.iter() {
        group.throughput(Throughput::Bytes(data.len() as u64));

        // Encode benchmarks
        group.bench_function(BenchmarkId::new("encode/hex", name), |b| {
            b.iter(|| hex::encode(black_box(data)))
        });

        group.bench_function(BenchmarkId::new("encode/muhex", name), |b| {
            b.iter(|| muhex::encode(black_box(data)))
        });

        group
            .bench_function(BenchmarkId::new("encode/faster-hex", name), |b| {
                b.iter(|| faster_hex::hex_string(black_box(*data)))
            });

        // Decode benchmarks
        let encoded = muhex::encode(data);
        group.bench_function(BenchmarkId::new("decode/hex", name), |b| {
            let mut output = vec![0; encoded.len() / 2];
            b.iter(|| {
                hex::decode_to_slice(
                    black_box(&encoded),
                    black_box(output.as_mut_slice()),
                )
                .unwrap()
            });
            black_box(output.as_slice());
        });
        group.bench_function(BenchmarkId::new("decode/muhex", name), |b| {
            let mut output = vec![0; encoded.len() / 2];
            b.iter(|| {
                muhex::decode_to_buf(
                    black_box(&encoded),
                    black_box(output.as_mut_slice()),
                )
                .unwrap();
                black_box(output.as_slice());
            })
        });
        group.bench_function(
            BenchmarkId::new("decode/faster-hex", name),
            |b| {
                let mut output = vec![0; encoded.len() / 2];
                b.iter(|| {
                    faster_hex::hex_decode(
                        black_box(encoded.as_bytes()),
                        black_box(&mut output),
                    )
                    .unwrap();
                    black_box(output.as_slice());
                })
            },
        );
    }

    group.finish();
}

#[cfg(feature = "serde")]
fn bench_serde(c: &mut Criterion) {
    let test_data = DATA_1MB.to_vec();

    let mut group = c.benchmark_group("serde");
    group.throughput(Throughput::Bytes(DATA_1MB.len() as u64));
    group.measurement_time(Duration::from_secs(5));

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
        muhex::serde::serialize(&test_data, serde_json::value::Serializer)
            .unwrap();

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

criterion_group!(benches, bench_compare_hex, bench_serde,);
criterion_main!(benches);
