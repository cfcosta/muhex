# Benchmarks

Generated from Criterion output in `target/criterion/encdec`.
Throughput is computed from `throughput.Bytes` and `new/estimates.json` mean time.

## Encode Throughput

| size | hex | muhex | faster-hex | muhex vs hex | muhex vs faster-hex |
| --- | ---: | ---: | ---: | ---: | ---: |
| `1MB_aligned` | 335.45 MiB/s | **34.69 GiB/s** | 23.69 GiB/s | 105.91x | 1.46x |
| `1KB_aligned` | 345.07 MiB/s | **18.05 GiB/s** | 14.48 GiB/s | 53.58x | 1.25x |
| `96B_aligned` | 310.81 MiB/s | **9.48 GiB/s** | 3.82 GiB/s | 31.23x | 2.48x |
| `64B_aligned` | 286.09 MiB/s | **6.80 GiB/s** | 1.90 GiB/s | 24.33x | 3.59x |
| `63B_unaligned` | 299.89 MiB/s | **6.41 GiB/s** | 1.60 GiB/s | 21.89x | 4.00x |
| `33B_unaligned` | 307.29 MiB/s | **3.35 GiB/s** | 1.58 GiB/s | 11.17x | 2.12x |
| `31B_remainder` | 301.91 MiB/s | **3.26 GiB/s** | 1.11 GiB/s | 11.07x | 2.94x |
| `17B_small` | 274.43 MiB/s | **1.81 GiB/s** | 868.78 MiB/s | 6.75x | 2.13x |
| `15B_small` | 250.62 MiB/s | **880.96 MiB/s** | 592.10 MiB/s | 3.52x | 1.49x |
| `7B_tiny` | 203.63 MiB/s | **549.34 MiB/s** | 329.78 MiB/s | 2.70x | 1.67x |
| `3B_tiny` | 121.34 MiB/s | **285.89 MiB/s** | 151.50 MiB/s | 2.36x | 1.89x |
| `1B_minimal` | 41.82 MiB/s | **111.10 MiB/s** | 52.75 MiB/s | 2.66x | 2.11x |

## Decode Throughput

| size | hex | muhex | faster-hex | muhex vs hex | muhex vs faster-hex |
| --- | ---: | ---: | ---: | ---: | ---: |
| `1MB_aligned` | 191.01 MiB/s | **38.06 GiB/s** | 9.09 GiB/s | 204.05x | 4.19x |
| `1KB_aligned` | 718.70 MiB/s | **48.52 GiB/s** | 8.80 GiB/s | 69.13x | 5.52x |
| `96B_aligned` | 778.13 MiB/s | **32.03 GiB/s** | 7.39 GiB/s | 42.16x | 4.34x |
| `64B_aligned` | 819.18 MiB/s | **28.95 GiB/s** | 6.44 GiB/s | 36.19x | 4.49x |
| `63B_unaligned` | 815.45 MiB/s | **21.45 GiB/s** | 2.76 GiB/s | 26.93x | 7.77x |
| `33B_unaligned` | 754.30 MiB/s | **11.17 GiB/s** | 4.25 GiB/s | 15.17x | 2.63x |
| `31B_remainder` | 815.27 MiB/s | **11.37 GiB/s** | 1.58 GiB/s | 14.28x | 7.19x |
| `17B_small` | 794.58 MiB/s | **6.26 GiB/s** | 1.37 GiB/s | 8.07x | 4.57x |
| `15B_small` | 766.48 MiB/s | **5.44 GiB/s** | 1.07 GiB/s | 7.26x | 5.08x |
| `7B_tiny` | 744.28 MiB/s | **1.15 GiB/s** | 641.94 MiB/s | 1.59x | 1.84x |
| `3B_tiny` | 595.30 MiB/s | **843.57 MiB/s** | 466.95 MiB/s | 1.42x | 1.81x |
| `1B_minimal` | **404.46 MiB/s** | 368.61 MiB/s | 244.05 MiB/s | 0.91x | 1.51x |
