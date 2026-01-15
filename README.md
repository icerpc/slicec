<p align="center">
  <img src="https://github.com/icerpc/slicec/raw/main/.github/assets/slicec-banner.svg" height="100" width="100" />
</p>

# The Slice compiler (slicec)

[![CI](https://github.com/icerpc/slicec/actions/workflows/ci.yml/badge.svg)][ci-home]
[![License](https://img.shields.io/github/license/icerpc/slicec?color=blue)][license]

## Targets

This crate has two targets, a library and a binary (both named `slicec`).

### `slicec` Library

The `slicec` library contains all the code for parsing and validating Slice definitions, taking either strings or files,
and converting them into a typed AST. It also exposes APIs for traversing and searching this AST, as well as retrieving
any warnings / errors discovered in the parsed Slice definitions.

Crates can include the `slicec` library by listing it as a dependency:

```toml
slicec = "0.3.3"
```

### `slicec` Binary

The `slicec` binary is a command-line tool which accepts Slice files (in addition to other flags), parses them into an
AST using the library, then writes a Slice-encoded version of the AST to `stdout`, for other tools to consume.

The `slicec` binary can be installed with:

```shell
cargo install slicec
```

It can also be run directly from source with:

```shell
cargo run
```

[ci-home]: https://github.com/icerpc/slicec/actions/workflows/ci.yml
[license]: https://github.com/icerpc/slicec/blob/main/LICENSE
