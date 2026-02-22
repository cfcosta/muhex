# Benchmarks

Generated from Criterion output in `target/criterion/encdec`.
Throughput is computed from `throughput.Bytes` and `new/estimates.json` mean time.

## Encode Throughput

| size | hex | muhex | faster-hex | speedup |
| --- | ---: | ---: | ---: | ---: |
| `1MB_aligned` | 342.69 MiB/s | **30.39 GiB/s** | 22.22 GiB/s | 90.81x |
| `1KB_aligned` | 342.17 MiB/s | **18.20 GiB/s** | 14.51 GiB/s | 54.47x |
| `96B_aligned` | 325.88 MiB/s | **8.26 GiB/s** | 3.23 GiB/s | 25.96x |
| `64B_aligned` | 325.18 MiB/s | **5.94 GiB/s** | 2.23 GiB/s | 18.69x |
| `63B_unaligned` | 335.22 MiB/s | **2.44 GiB/s** | 1.63 GiB/s | 7.45x |
| `33B_unaligned` | 332.04 MiB/s | **2.92 GiB/s** | 1.55 GiB/s | 9.02x |
| `31B_remainder` | 309.15 MiB/s | **1.29 GiB/s** | 1.13 GiB/s | 4.26x |
| `17B_small` | 292.48 MiB/s | **1.56 GiB/s** | 854.14 MiB/s | 5.46x |
| `15B_small` | 281.08 MiB/s | **654.38 MiB/s** | 590.02 MiB/s | 2.33x |
| `7B_tiny` | 189.74 MiB/s | **458.80 MiB/s** | 323.77 MiB/s | 2.42x |
| `3B_tiny` | 116.40 MiB/s | **267.29 MiB/s** | 151.53 MiB/s | 2.30x |
| `1B_minimal` | 39.54 MiB/s | **103.58 MiB/s** | 52.36 MiB/s | 2.62x |

## Decode Throughput

| size | hex | muhex | faster-hex | speedup |
| --- | ---: | ---: | ---: | ---: |
| `1MB_aligned` | 193.03 MiB/s | **18.57 GiB/s** | 8.62 GiB/s | 98.50x |
| `1KB_aligned` | 796.22 MiB/s | **18.28 GiB/s** | 8.60 GiB/s | 23.50x |
| `96B_aligned` | 824.45 MiB/s | **13.22 GiB/s** | 7.13 GiB/s | 16.42x |
| `64B_aligned` | 825.36 MiB/s | **12.85 GiB/s** | 6.27 GiB/s | 15.95x |
| `63B_unaligned` | 832.15 MiB/s | **4.28 GiB/s** | 2.75 GiB/s | 5.26x |
| `33B_unaligned` | 821.27 MiB/s | **8.62 GiB/s** | 4.20 GiB/s | 10.75x |
| `31B_remainder` | 817.48 MiB/s | **2.38 GiB/s** | 1.55 GiB/s | 2.98x |
| `17B_small` | 804.39 MiB/s | **4.21 GiB/s** | 1.35 GiB/s | 5.36x |
| `15B_small` | 805.97 MiB/s | **1.33 GiB/s** | 1.04 GiB/s | 1.70x |
| `7B_tiny` | 778.67 MiB/s | **1.14 GiB/s** | 631.76 MiB/s | 1.49x |
| `3B_tiny` | 641.64 MiB/s | **848.19 MiB/s** | 463.97 MiB/s | 1.32x |
| `1B_minimal` | **532.45 MiB/s** | 398.99 MiB/s | 238.40 MiB/s | 1.00x |
