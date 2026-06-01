<p align="center">
  <img src="https://github.com/icerpc/slicec/raw/main/.github/assets/slicec-banner.svg" height="100" width="100" />
</p>

# Slice for Rust

[![CI](https://github.com/icerpc/slicec/actions/workflows/ci.yml/badge.svg)][ci-badge]
[![License](https://img.shields.io/github/license/icerpc/slicec?color=blue)][license]

This repository is home to the Rust crates for working with [Slice][slice], the interface definition language (IDL)
used by [IceRPC][icerpc]. It is a Cargo workspace made up of the following crates:

| Crate                          | Description                                                                                  | crates.io                                       |
| ------------------------------ | -------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| [`slicec`](./slicec)           | The Slice parser and compiler. Ships as both a library and the `slicec` command-line binary. | [![slicec][slicec-badge]][slicec-crate]         |
| [`slice-codec`](./slice-codec) | A lightweight, `no_std`-friendly library for encoding and decoding Slice-encoded data.       | [![slice-codec][codec-badge]][codec-crate]      |

See each crate's README for more specific details.

## Building

To build everything in the workspace you'll need [Rust and Cargo][rust-install] installed.
From the root of the repository, run:

```shell
cargo build
```

To run the full test suite for every crate in the workspace, run:

```shell
cargo test
```

To generate documentation for every crate in the workspace, run:

```shell
cargo doc --no-deps --document-private-items
```

## Documentation

- Slice language documentation: <https://docs.icerpc.dev/slice>
- `slicec` API docs: <https://docs.rs/slicec>
- `slice-codec` API docs: <https://docs.rs/slice-codec>

## License

Licensed under the [Apache License, Version 2.0][license].

[ci-badge]: https://github.com/icerpc/slicec/actions/workflows/ci.yml
[license]: https://github.com/icerpc/slicec/blob/main/LICENSE
[slice]: https://docs.icerpc.dev/slice
[icerpc]: https://github.com/icerpc/icerpc-csharp
[rust-install]: https://doc.rust-lang.org/cargo/getting-started/installation.html
[slicec-crate]: https://crates.io/crates/slicec
[slicec-badge]: https://img.shields.io/crates/v/slicec?color=blue
[codec-crate]: https://crates.io/crates/slice-codec
[codec-badge]: https://img.shields.io/crates/v/slice-codec?color=blue
