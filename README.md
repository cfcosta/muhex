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

If you already own a buffer, you can decode without any allocation:

```rust
let mut buf = vec![0u8; 12];
muhex::decode_to_slice("48656c6c6f20776f726c6421", &mut buf)?;
```

## Benchmarks

This is a benchmark on my own machine against the `hex` and
[`faster-hex`](https://crates.io/crates/faster-hex) crates.

* Command: `RUSTFLAGS="-C target-cpu=native -C target-feature=+avx2,+avx,+sse2" cargo bench --all-features`
* Machine: Ryzen 7950X3D 128GB DDR5 RAM

| Operation | Implementation | Time     | Throughput    | Speedup vs `hex` |
|-----------|----------------|----------|---------------|------------------|
| Encode    | hex            | 2.690 ms | 371.77 MiB/s  | 1x               |
| Encode    | muhex          | 47.72 µs | 20.47 GiB/s   | ~56x             |
| Encode    | faster-hex     | 43.67 µs | 22.36 GiB/s   | ~62x             |
| Decode    | hex            | 6.437 ms | 155.36 MiB/s  | 1x               |
| Decode    | muhex          | 66.10 µs | 14.77 GiB/s   | ~97x             |
| Decode    | faster-hex     | 64.81 µs | 15.07 GiB/s   | ~99x             |

Please note that we can only achieve this performance because we only work on nightly Rust and explicitly enforce SIMD. This is not a statement about the quality or performance of the competing crates—most applications may not benefit from any of these changes.
