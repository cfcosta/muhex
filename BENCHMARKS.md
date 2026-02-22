# Benchmarks

Generated from Criterion output in `target/criterion/encdec`.
Throughput is computed from `throughput.Bytes` and `new/estimates.json` mean time.

## Encode Throughput

| size | hex | muhex | faster-hex | speedup |
| --- | ---: | ---: | ---: | ---: |
| `1MB_aligned` | 341.71 MiB/s | **29.33 GiB/s** | 20.98 GiB/s | 87.90x |
| `1KB_aligned` | 350.90 MiB/s | **16.31 GiB/s** | 11.19 GiB/s | 47.59x |
| `96B_aligned` | 341.50 MiB/s | **8.39 GiB/s** | 3.78 GiB/s | 25.14x |
| `64B_aligned` | 328.45 MiB/s | **5.97 GiB/s** | 2.25 GiB/s | 18.60x |
| `63B_unaligned` | 320.38 MiB/s | **5.73 GiB/s** | 1.73 GiB/s | 18.32x |
| `33B_unaligned` | 311.65 MiB/s | **2.98 GiB/s** | 1.55 GiB/s | 9.79x |
| `31B_remainder` | 319.12 MiB/s | **3.06 GiB/s** | 1.13 GiB/s | 9.80x |
| `17B_small` | 279.92 MiB/s | **1.68 GiB/s** | 854.39 MiB/s | 6.13x |
| `15B_small` | 278.62 MiB/s | **651.60 MiB/s** | 577.68 MiB/s | 2.34x |
| `7B_tiny` | 198.80 MiB/s | **456.37 MiB/s** | 320.13 MiB/s | 2.30x |
| `3B_tiny` | 118.25 MiB/s | **268.10 MiB/s** | 150.87 MiB/s | 2.27x |
| `1B_minimal` | 40.56 MiB/s | **105.86 MiB/s** | 52.51 MiB/s | 2.61x |

## Decode Throughput

| size | hex | muhex | faster-hex | speedup |
| --- | ---: | ---: | ---: | ---: |
| `1MB_aligned` | 183.37 MiB/s | **18.61 GiB/s** | 8.72 GiB/s | 103.94x |
| `1KB_aligned` | 726.65 MiB/s | **19.03 GiB/s** | 8.64 GiB/s | 26.82x |
| `96B_aligned` | 775.48 MiB/s | **14.98 GiB/s** | 7.18 GiB/s | 19.79x |
| `64B_aligned` | 778.76 MiB/s | **15.52 GiB/s** | 6.32 GiB/s | 20.41x |
| `63B_unaligned` | 777.23 MiB/s | **13.81 GiB/s** | 2.71 GiB/s | 18.19x |
| `33B_unaligned` | 772.68 MiB/s | **7.22 GiB/s** | 4.11 GiB/s | 9.57x |
| `31B_remainder` | 770.33 MiB/s | **7.15 GiB/s** | 1.54 GiB/s | 9.51x |
| `17B_small` | 764.60 MiB/s | **3.95 GiB/s** | 1.33 GiB/s | 5.29x |
| `15B_small` | 756.77 MiB/s | **1.28 GiB/s** | 1.06 GiB/s | 1.74x |
| `7B_tiny` | 728.79 MiB/s | **1.08 GiB/s** | 667.16 MiB/s | 1.51x |
| `3B_tiny` | 682.36 MiB/s | **787.51 MiB/s** | 475.93 MiB/s | 1.15x |
| `1B_minimal` | **526.13 MiB/s** | 396.84 MiB/s | 238.16 MiB/s | 1.00x |
