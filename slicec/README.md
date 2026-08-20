<p align="center">
  <img src="https://github.com/icerpc/slicec/raw/main/.github/assets/slicec-banner.svg" height="100" width="100" />
</p>

# The Slice compiler (slicec)

[![CI](https://github.com/icerpc/slicec/actions/workflows/ci.yml/badge.svg)][ci-badge]
[![crates.io](https://img.shields.io/crates/v/slicec?color=blue)][crates-home]
[![docs.rs](https://img.shields.io/docsrs/slicec)][docs-home]
[![License](https://img.shields.io/github/license/icerpc/slicec?color=blue)](./LICENSE)

`slicec` is the compiler for [Slice][slice], the _Interface Definition Language_ (IDL) used by [IceRPC][icerpc].

This crate provides two targets — a **binary** and a **library** — both named `slicec`.

## `slicec` binary

The `slicec` binary is a command-line tool that takes Slice files, parses them into a typed _Abstract Syntax Tree_
(AST), validates them, and then encodes them using the [Slice encoding][slice-encoding] so other tools can consume it.

Code generation is performed by **generator plugins**: external executables that are passed with `--generator` (or `-G`)
on the command line. `slicec` spawns each generator as a subprocess, sends the Slice-encoded AST over its `stdin`, and
waits for a response over its `stdout`. This response consists of generated files (which `slicec` will then write to the
appropriate location) and diagnostics (which are collected and reported by `slicec` at the end).

### Installation

To install the `slicec` binary from `crates.io`, run:

```shell
cargo install slicec
```

### Usage

To see the full list of options and flags supported by `slicec`, run `slicec --help`.

```shell
# Parse and validate one Slice file, and run two code-generator plugins over the result.
slicec hello.slice --generator /path/to/my/generator --generator my_other_generator

# Parses and validates 'project/hello.slice' and any reference files in './references'.
# Code-generator plugins only generate code for non-reference files.
slicec project/hello.slice -R ./references --generator /path/to/my/generator

# Validate and parse Slice files without generating any code.
slicec hello.slice goodbye.slice -R ./reference-files
```

If you have the source code checked out locally, you can also run the binary using cargo:

```shell
# Run `slicec` directly from source.
cargo run -- hello.slice --generator /path/to/my/generator
```

## `slicec` library

The `slicec` library contains all the logic for parsing input into typed definitions stored in a single AST. It:

- Parses Slice definitions from on-disk files or in-memory strings.
- Validates definitions and reports findings through structured diagnostics.
- Exposes APIs for traversing and searching through the AST and definitions within it.

### Using `slicec` in a Cargo Project:

Add `slicec` as a dependency to your project's `Cargo.toml` file:

```toml
[dependencies]
slicec = "0.5"
```

### Example

```rust
use slicec::compile_from_strings;
use slicec::grammar::Enum;

let source = "
    module Example

    enum Color {
        Hex(color: string)
        Rgb(red: uint8, green: uint8, blue: uint8, tag(1) alpha: uint8?)
        Hsl(hue: int32, saturation: float32, lightness: float32)
    }
";

// Compile the Slice definitions. `None` means "use the default compiler options".
let compilation_state = compile_from_strings(&[source], None);

// Make sure no diagnostics were reported.
assert!(!compilation_state.diagnostics.has_errors());
// Inspect the contents of the resulting AST.
let color_enum = compilation_state.ast.find_element::<Enum>("Example::Color").unwrap();
```

## Building

This crate is part of the [`slicec` workspace](https://github.com/icerpc/slicec).
Commands should be run from the repository root:

```shell
# Build with the default features.
cargo build -p slicec

# Run the tests.
cargo test -p slicec
```

## License

'slicec' is licensed under the [Apache License, Version 2.0](./LICENSE).

[ci-badge]: https://github.com/icerpc/slicec/actions/workflows/ci.yml
[crates-home]: https://crates.io/crates/slicec
[docs-home]: https://docs.rs/slicec
[slice]: https://docs.icerpc.dev/slice
[slice-encoding]: https://docs.icerpc.dev/slice/encoding/overview
[icerpc]: https://github.com/icerpc/icerpc-csharp
