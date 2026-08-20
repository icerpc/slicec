<p align="center">
  <img src="https://github.com/icerpc/slicec/raw/main/.github/assets/slicec-banner.svg" height="100" width="100" />
</p>

# The Slice codec (slice-codec)

[![CI](https://github.com/icerpc/slicec/actions/workflows/ci.yml/badge.svg)][ci-badge]
[![crates.io](https://img.shields.io/crates/v/slice-codec?color=blue)][crates-home]
[![docs.rs](https://img.shields.io/docsrs/slice-codec)][docs-home]
[![License](https://img.shields.io/github/license/icerpc/slicec?color=blue)](./LICENSE)

`slice-codec` is a lightweight library for encoding and decoding values using the [Slice encoding][slice-encoding].

## Overview

The codec is built around two pairs of types:

- `Encoder` writes values into an `OutputTarget`, driven by the `EncodeInto` trait.
- `Decoder` reads values out of an `InputSource`, driven by the `DecodeFrom` trait.

Implementations of `EncodeInto` / `DecodeFrom` are provided for common primitives and collections,
but you can implement these traits on your own types to make them usable with [Slice][slice-encoding].

### Example

```rust
use slice_codec::encoder::Encoder;
use slice_codec::decoder::Decoder;

fn main() -> slice_codec::Result<()> {
    let mut buffer: Vec<u8> = Vec::new();

    // Encode some values into a byte buffer.
    let mut encoder = Encoder::from(&mut buffer);
    encoder.encode("hello")?;
    encoder.encode::<i32>(42)?;

    // Decode them back out.
    let mut decoder = Decoder::from(&buffer);
    let greeting: String = decoder.decode()?;
    let the_answer = decoder.decode::<i32>()?;

    assert_eq!(greeting, "hello");
    assert_eq!(the_answer, 42);
    Ok(())
}
```

## Using `slice-codec` in a Cargo Project:

Add `slice-codec` as a dependency to your project's `Cargo.toml` file:

```toml
[dependencies]
slice-codec = "0.5"
```

### Usage in `no_std` Projects

The codec is `no_std` compatible, so it can be used in embedded or other constrained environments.  
To use the codec in a `no_std` project, just disable default features:

```toml
[dependencies]
slice-codec = { version = "0.5", default-features = false }
```

If you have a global allocator and need to decode owned types like `String` or `Vec`, also enable the `alloc` feature.

### Feature Flags

| Feature  | Default | Description                                                                                                                      |
| -------- | :-----: | ---------------------------------------------------------------------------------------------------------------------------------|
| `std`    |   ✅     | Enables implementations to encode/decode standard-library types such as `String`, `Vec`, and `HashMap`. Implies `alloc`.         |
| `alloc`  |   ✅     | Enables implementations to decode dynamically-allocated types like `String` and `Vec`. Intended for `no_std` + global allocator. |

## Building

This crate is part of the [`slicec` workspace](https://github.com/icerpc/slicec).
Commands should be run from the repository root:

```shell
# Build with the default features.
cargo build -p slice-codec

# Run the tests with all features enabled.
cargo test -p slice-codec --all-features
```

## License

'slicec-codec' is licensed under the [Apache License, Version 2.0](./LICENSE).

[ci-badge]: https://github.com/icerpc/slicec/actions/workflows/ci.yml
[crates-home]: https://crates.io/crates/slice-codec
[docs-home]: https://docs.rs/slice-codec
[slice-encoding]: https://docs.icerpc.dev/slice/encoding/overview
