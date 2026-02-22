# Benchmarks

Generated from Criterion output in `target/criterion/encdec`.
Throughput is computed from `throughput.Bytes` and `new/estimates.json` mean time.

## Encode Throughput

| size | hex | muhex | faster-hex | muhex vs hex | muhex vs faster-hex |
| --- | ---: | ---: | ---: | ---: | ---: |
| `1MB_aligned` | 336.16 MiB/s | **30.10 GiB/s** | 22.19 GiB/s | 91.68x | 1.36x |
| `1KB_aligned` | 333.04 MiB/s | 14.17 GiB/s | **14.86 GiB/s** | 43.56x | 0.95x |
| `96B_aligned` | 326.71 MiB/s | **8.32 GiB/s** | 3.18 GiB/s | 26.09x | 2.62x |
| `64B_aligned` | 312.58 MiB/s | **5.86 GiB/s** | 2.23 GiB/s | 19.20x | 2.63x |
| `63B_unaligned` | 331.51 MiB/s | **5.66 GiB/s** | 1.57 GiB/s | 17.47x | 3.60x |
| `33B_unaligned` | 307.76 MiB/s | **3.00 GiB/s** | 1.52 GiB/s | 9.98x | 1.97x |
| `31B_remainder` | 317.75 MiB/s | **3.03 GiB/s** | 1.13 GiB/s | 9.76x | 2.69x |
| `17B_small` | 280.33 MiB/s | **1.64 GiB/s** | 832.21 MiB/s | 5.98x | 2.02x |
| `15B_small` | 269.28 MiB/s | **638.93 MiB/s** | 570.22 MiB/s | 2.37x | 1.12x |
| `7B_tiny` | 190.87 MiB/s | **448.99 MiB/s** | 309.54 MiB/s | 2.35x | 1.45x |
| `3B_tiny` | 115.61 MiB/s | **257.90 MiB/s** | 144.35 MiB/s | 2.23x | 1.79x |
| `1B_minimal` | 39.75 MiB/s | **103.47 MiB/s** | 51.09 MiB/s | 2.60x | 2.03x |

## Decode Throughput

| size | hex | muhex | faster-hex | muhex vs hex | muhex vs faster-hex |
| --- | ---: | ---: | ---: | ---: | ---: |
| `1MB_aligned` | 182.65 MiB/s | **18.61 GiB/s** | 8.57 GiB/s | 104.32x | 2.17x |
| `1KB_aligned` | 699.45 MiB/s | **19.32 GiB/s** | 8.45 GiB/s | 28.28x | 2.29x |
| `96B_aligned` | 773.19 MiB/s | **17.21 GiB/s** | 7.05 GiB/s | 22.80x | 2.44x |
| `64B_aligned` | 782.37 MiB/s | **15.13 GiB/s** | 6.24 GiB/s | 19.80x | 2.42x |
| `63B_unaligned` | 774.62 MiB/s | **13.94 GiB/s** | 2.69 GiB/s | 18.43x | 5.19x |
| `33B_unaligned` | 757.40 MiB/s | **7.13 GiB/s** | 4.04 GiB/s | 9.64x | 1.76x |
| `31B_remainder` | 772.47 MiB/s | **7.12 GiB/s** | 1.52 GiB/s | 9.44x | 4.68x |
| `17B_small` | 748.17 MiB/s | **3.90 GiB/s** | 1.31 GiB/s | 5.34x | 2.97x |
| `15B_small` | 758.01 MiB/s | **1.28 GiB/s** | 1.07 GiB/s | 1.72x | 1.20x |
| `7B_tiny` | 715.79 MiB/s | **1.04 GiB/s** | 651.96 MiB/s | 1.49x | 1.64x |
| `3B_tiny` | 652.33 MiB/s | **770.07 MiB/s** | 470.81 MiB/s | 1.18x | 1.64x |
| `1B_minimal` | **508.30 MiB/s** | 386.92 MiB/s | 236.24 MiB/s | 0.76x | 1.64x |
