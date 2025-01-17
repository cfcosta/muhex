# muhex

> [!WARNING]
> `muhex` requires a nightly version of the Rust compiler.

Muhex provides encoding and decoding in a hexadecimal representation, focusing on speed. It has zero dependencies (one optional, `serde`), compiles instantly and is faster.

## Usage

Install the crate using the normal incantations:

```sh
cargo +nightly add muhex
```

Then, you can use it as expected, and the interface is meant to mirror the one on the [`hex`](https://docs.rs/hex) crate, like encoding:

```rust
println!("{}", muhex::encode("Hello world!")); // Prints "48656c6c6f20776f726c6421"
```

And decoding:

```rust
println!("{}", muhex::decode("48656c6c6f20776f726c6421")?); // Prints "Hello world!"
```

## Benchmarks

Command: `RUSTFLAGS="-C target-cpu=native -C target-feature=+avx2,+avx,+sse2" cargo bench --all-feature`
Machine: Ryzen 7950X3D 128GB DDR5 RAM

```
encode/decode 1M/encode/hex
                        time:   [2.6163 ms 2.6413 ms 2.6640 ms]
                        thrpt:  [375.38 MiB/s 378.60 MiB/s 382.21 MiB/s]
encode/decode 1M/encode/muhex
                        time:   [51.094 µs 51.213 µs 51.337 µs]
                        thrpt:  [19.023 GiB/s 19.069 GiB/s 19.113 GiB/s]
encode/decode 1M/decode/hex
                        time:   [6.0813 ms 6.0959 ms 6.1117 ms]
                        thrpt:  [163.62 MiB/s 164.04 MiB/s 164.44 MiB/s]
encode/decode 1M/decode/muhex
                        time:   [4.7397 ms 4.7804 ms 4.8267 ms]
                        thrpt:  [207.18 MiB/s 209.19 MiB/s 210.98 MiB/s]
serde/serialize/muhex   time:   [1.0163 ms 1.0214 ms 1.0275 ms]
                        thrpt:  [973.25 MiB/s 979.01 MiB/s 983.95 MiB/s]
serde/serialize/hex     time:   [3.3410 ms 3.3546 ms 3.3733 ms]
                        thrpt:  [296.45 MiB/s 298.10 MiB/s 299.31 MiB/s]
serde/deserialize/muhex time:   [4.6475 ms 4.6671 ms 4.6876 ms]
                        thrpt:  [213.33 MiB/s 214.26 MiB/s 215.17 MiB/s]
Benchmarking serde/deserialize/hex: Warming up for 3.0000 s
serde/deserialize/hex   time:   [6.2650 ms 6.2924 ms 6.3197 ms]
                        thrpt:  [158.23 MiB/s 158.92 MiB/s 159.62 MiB/s]
```
