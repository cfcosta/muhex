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

This is a benchmark on my own machine against the `hex` crate.

* Command: `RUSTFLAGS="-C target-cpu=native -C target-feature=+avx2,+avx,+sse2" cargo bench --all-feature`
* Machine: Ryzen 7950X3D 128GB DDR5 RAM

| Operation    | Implementation | Time      | Throughput    | Speedup |
|-------------|----------------|-----------|---------------|---------|
| Encode      | hex            | 3.089 ms  | 323.67 MiB/s  | 1x      |
| Encode      | muhex          | 52.23 µs  | 18.696 GiB/s  | ~57x    |
| Decode      | hex            | 6.172 ms  | 162.03 MiB/s  | 1x      |
| Decode      | muhex          | 160.97 µs | 6.067 GiB/s   | ~38x    |
| Serialize   | hex            | 4.404 ms  | 227.04 MiB/s  | 1x      |
| Serialize   | muhex          | 968.59 µs | 1.008 GiB/s   | ~4.5x   |
| Deserialize | hex            | 6.341 ms  | 157.71 MiB/s  | 1x      |
| Deserialize | muhex          | 166.55 µs | 5.864 GiB/s   | ~38x    |

Please note that we can only achieve this performance because we only work on nightly Rust, and explicitly enforce SIMD. This is not a testament on the quality or performance of the `hex` crate. In fact, in most applications (if not all) will not benefit from any of those changes.
