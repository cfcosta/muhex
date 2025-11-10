use std::time::Duration;

use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
    Throughput,
};

const DATA_1MB: &[u8; 1024 * 1024] = include_bytes!("seed.bin");

fn bench_compare_hex(c: &mut Criterion) {
    let mut group = c.benchmark_group("encdec");
    group.measurement_time(Duration::from_secs(30));

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

// Benchmark to compare LUT vs SIMD remainder strategies directly
#[cfg(feature = "test-util")]
fn bench_remainder_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("remainder_strategy");
    group.measurement_time(Duration::from_secs(20));

    // Test pure remainder processing (no main loop)
    let pure_remainder_sizes = [
        ("2B_1pair", 2),
        ("4B_2pairs", 4),
        ("6B_3pairs", 6),
        ("8B_4pairs", 8),
        ("12B_6pairs", 12),
        ("16B_8pairs", 16),
        ("20B_10pairs", 20),
        ("24B_12pairs", 24),
        ("30B_15pairs", 30),
    ];

    for (name, hex_len) in pure_remainder_sizes {
        let data = &DATA_1MB[..hex_len / 2];
        let encoded = muhex::encode(data);
        let input_bytes = encoded.as_bytes();

        group.throughput(Throughput::Bytes(data.len() as u64));

        // Benchmark SIMD remainder directly
        group.bench_function(BenchmarkId::new("simd", name), |b| {
            let mut output = vec![std::mem::MaybeUninit::uninit(); data.len()];
            b.iter(|| {
                muhex::decode_remainder_simd_bench(
                    black_box(input_bytes),
                    black_box(&mut output),
                    0,
                    0,
                    input_bytes.len(),
                )
                .unwrap();
                black_box(output.as_slice());
            })
        });

        // Benchmark LUT remainder directly
        group.bench_function(BenchmarkId::new("lut", name), |b| {
            let mut output = vec![std::mem::MaybeUninit::uninit(); data.len()];
            b.iter(|| {
                muhex::decode_remainder_lut_bench(
                    black_box(input_bytes),
                    black_box(&mut output),
                    0,
                    0,
                    input_bytes.len(),
                )
                .unwrap();
                black_box(output.as_slice());
            })
        });
    }

    group.finish();
}

// More detailed remainder comparison with warmup
#[cfg(feature = "test-util")]
fn bench_remainder_detailed(c: &mut Criterion) {
    let mut group = c.benchmark_group("remainder_detailed");
    group.measurement_time(Duration::from_secs(30));
    group.warm_up_time(Duration::from_secs(5));

    // Pre-allocate output buffers to avoid allocation overhead
    let sizes = [
        ("1_pair", 1),
        ("2_pairs", 2),
        ("3_pairs", 3),
        ("4_pairs", 4),
        ("5_pairs", 5),
        ("6_pairs", 6),
        ("7_pairs", 7),
        ("8_pairs", 8),
        ("9_pairs", 9),
        ("10_pairs", 10),
        ("11_pairs", 11),
        ("12_pairs", 12),
        ("13_pairs", 13),
        ("14_pairs", 14),
        ("15_pairs", 15),
    ];

    for (name, pairs) in sizes {
        let byte_len = pairs;
        let data = &DATA_1MB[..byte_len];
        let encoded = muhex::encode(data);
        let input_bytes = encoded.as_bytes();

        group.throughput(Throughput::Bytes(byte_len as u64));

        // SIMD with pre-allocated buffer
        group.bench_function(BenchmarkId::new("simd", name), |b| {
            let mut output = vec![std::mem::MaybeUninit::uninit(); byte_len];
            b.iter(|| {
                muhex::decode_remainder_simd_bench(
                    black_box(input_bytes),
                    black_box(&mut output),
                    0,
                    0,
                    input_bytes.len(),
                )
                .unwrap();
                black_box(output.as_slice());
            })
        });

        // LUT with pre-allocated buffer
        group.bench_function(BenchmarkId::new("lut", name), |b| {
            let mut output = vec![std::mem::MaybeUninit::uninit(); byte_len];
            b.iter(|| {
                muhex::decode_remainder_lut_bench(
                    black_box(input_bytes),
                    black_box(&mut output),
                    0,
                    0,
                    input_bytes.len(),
                )
                .unwrap();
                black_box(output.as_slice());
            })
        });
    }

    group.finish();
}

#[cfg(feature = "serde")]
fn bench_serde(c: &mut Criterion) {
    let test_data = DATA_1MB.to_vec();

    let mut group = c.benchmark_group("serde");
    group.throughput(Throughput::Bytes(DATA_1MB.len() as u64));
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

#[cfg(not(feature = "test-util"))]
fn bench_remainder_strategies(_c: &mut Criterion) {}

#[cfg(not(feature = "test-util"))]
fn bench_remainder_detailed(_c: &mut Criterion) {}

criterion_group!(
    benches,
    bench_compare_hex,
    bench_serde,
    bench_remainder_strategies,
    bench_remainder_detailed,
);
criterion_main!(benches);
