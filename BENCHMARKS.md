# Benchmarks

Generated from Criterion output in `target/criterion/encdec`.
Throughput is computed from `throughput.Bytes` and `new/estimates.json` mean time.

## Encode Throughput

| size | hex | muhex | faster-hex | muhex vs hex | muhex vs faster-hex |
| --- | ---: | ---: | ---: | ---: | ---: |
| `1MB_aligned` | 236.35 MiB/s | **37.70 GiB/s** | 21.89 GiB/s | 163.32x | 1.72x |
| `1KB_aligned` | 288.14 MiB/s | **18.36 GiB/s** | 12.17 GiB/s | 65.24x | 1.51x |
| `96B_aligned` | 275.60 MiB/s | **8.81 GiB/s** | 3.18 GiB/s | 32.73x | 2.77x |
| `64B_aligned` | 269.51 MiB/s | **6.39 GiB/s** | 2.01 GiB/s | 24.30x | 3.19x |
| `63B_unaligned` | 266.30 MiB/s | **5.96 GiB/s** | 1.70 GiB/s | 22.91x | 3.51x |
| `33B_unaligned` | 256.49 MiB/s | **3.08 GiB/s** | 1.51 GiB/s | 12.31x | 2.05x |
| `31B_remainder` | 254.65 MiB/s | **3.11 GiB/s** | 1.10 GiB/s | 12.53x | 2.84x |
| `17B_small` | 231.18 MiB/s | **1.69 GiB/s** | 810.41 MiB/s | 7.47x | 2.13x |
| `15B_small` | 227.46 MiB/s | **840.08 MiB/s** | 569.61 MiB/s | 3.69x | 1.47x |
| `7B_tiny` | 181.28 MiB/s | **521.45 MiB/s** | 310.25 MiB/s | 2.88x | 1.68x |
| `3B_tiny` | 117.98 MiB/s | **277.34 MiB/s** | 143.74 MiB/s | 2.35x | 1.93x |
| `1B_minimal` | 37.74 MiB/s | **105.12 MiB/s** | 50.79 MiB/s | 2.79x | 2.07x |

## Decode Throughput

| size | hex | muhex | faster-hex | muhex vs hex | muhex vs faster-hex |
| --- | ---: | ---: | ---: | ---: | ---: |
| `1MB_aligned` | 186.06 MiB/s | **16.11 GiB/s** | 8.54 GiB/s | 88.69x | 1.89x |
| `1KB_aligned` | 746.10 MiB/s | **16.56 GiB/s** | 8.61 GiB/s | 22.73x | 1.92x |
| `96B_aligned` | 777.84 MiB/s | **16.57 GiB/s** | 7.08 GiB/s | 21.82x | 2.34x |
| `64B_aligned` | 785.78 MiB/s | **13.01 GiB/s** | 6.22 GiB/s | 16.95x | 2.09x |
| `63B_unaligned` | 774.61 MiB/s | **14.29 GiB/s** | 2.66 GiB/s | 18.89x | 5.38x |
| `33B_unaligned` | 771.45 MiB/s | **8.02 GiB/s** | 4.05 GiB/s | 10.64x | 1.98x |
| `31B_remainder` | 779.93 MiB/s | **8.53 GiB/s** | 1.46 GiB/s | 11.20x | 5.84x |
| `17B_small` | 743.99 MiB/s | **4.65 GiB/s** | 1.30 GiB/s | 6.40x | 3.58x |
| `15B_small` | 763.01 MiB/s | **1.33 GiB/s** | 1.03 GiB/s | 1.79x | 1.29x |
| `7B_tiny` | 711.55 MiB/s | **1.15 GiB/s** | 695.94 MiB/s | 1.66x | 1.69x |
| `3B_tiny` | 591.89 MiB/s | **919.77 MiB/s** | 463.40 MiB/s | 1.55x | 1.98x |
| `1B_minimal` | 419.19 MiB/s | **513.55 MiB/s** | 231.16 MiB/s | 1.23x | 2.22x |
