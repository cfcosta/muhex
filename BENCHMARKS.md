# Benchmarks

Generated from Criterion output in `target/criterion/encdec`.
Throughput is computed from `throughput.Bytes` and `new/estimates.json` mean time.

## Encode Throughput

| size | hex | muhex | faster-hex | muhex vs hex | muhex vs faster-hex |
| --- | ---: | ---: | ---: | ---: | ---: |
| `1MB_aligned` | 294.63 MiB/s | **34.17 GiB/s** | 21.82 GiB/s | 118.77x | 1.57x |
| `1KB_aligned` | 296.93 MiB/s | **21.19 GiB/s** | 14.76 GiB/s | 73.08x | 1.44x |
| `96B_aligned` | 285.19 MiB/s | **9.03 GiB/s** | 3.32 GiB/s | 32.43x | 2.72x |
| `64B_aligned` | 278.56 MiB/s | **4.18 GiB/s** | 2.06 GiB/s | 15.37x | 2.03x |
| `63B_unaligned` | 279.87 MiB/s | **4.12 GiB/s** | 1.94 GiB/s | 15.08x | 2.13x |
| `33B_unaligned` | 266.98 MiB/s | **3.22 GiB/s** | 1.56 GiB/s | 12.34x | 2.06x |
| `31B_remainder` | 263.97 MiB/s | **3.25 GiB/s** | 1.14 GiB/s | 12.60x | 2.85x |
| `17B_small` | 241.61 MiB/s | **1.78 GiB/s** | 854.59 MiB/s | 7.55x | 2.13x |
| `15B_small` | 233.89 MiB/s | **867.14 MiB/s** | 588.27 MiB/s | 3.71x | 1.47x |
| `7B_tiny` | 182.96 MiB/s | **543.70 MiB/s** | 325.10 MiB/s | 2.97x | 1.67x |
| `3B_tiny` | 117.16 MiB/s | **285.59 MiB/s** | 147.53 MiB/s | 2.44x | 1.94x |
| `1B_minimal` | 39.36 MiB/s | **107.40 MiB/s** | 52.43 MiB/s | 2.73x | 2.05x |

## Decode Throughput

| size | hex | muhex | faster-hex | muhex vs hex | muhex vs faster-hex |
| --- | ---: | ---: | ---: | ---: | ---: |
| `1MB_aligned` | 185.22 MiB/s | **35.39 GiB/s** | 8.67 GiB/s | 195.68x | 4.08x |
| `1KB_aligned` | 730.07 MiB/s | **48.87 GiB/s** | 8.68 GiB/s | 68.54x | 5.63x |
| `96B_aligned` | 795.73 MiB/s | **11.77 GiB/s** | 7.29 GiB/s | 15.14x | 1.62x |
| `64B_aligned` | 792.16 MiB/s | **26.12 GiB/s** | 6.41 GiB/s | 33.77x | 4.07x |
| `63B_unaligned` | 794.07 MiB/s | **21.28 GiB/s** | 2.74 GiB/s | 27.44x | 7.76x |
| `33B_unaligned` | 778.79 MiB/s | **10.57 GiB/s** | 4.20 GiB/s | 13.90x | 2.52x |
| `31B_remainder` | 787.81 MiB/s | **11.25 GiB/s** | 1.56 GiB/s | 14.62x | 7.19x |
| `17B_small` | 775.66 MiB/s | **6.15 GiB/s** | 1.33 GiB/s | 8.12x | 4.62x |
| `15B_small` | 768.59 MiB/s | **5.45 GiB/s** | 1.01 GiB/s | 7.26x | 5.39x |
| `7B_tiny` | 736.88 MiB/s | **1.14 GiB/s** | 641.94 MiB/s | 1.58x | 1.82x |
| `3B_tiny` | 683.83 MiB/s | **829.69 MiB/s** | 468.48 MiB/s | 1.21x | 1.77x |
| `1B_minimal` | **530.57 MiB/s** | 388.19 MiB/s | 239.12 MiB/s | 0.73x | 1.62x |
