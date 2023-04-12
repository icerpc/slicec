# Slice Compiler Library (slicec)

[![.github/workflows/rust.yml](https://github.com/icerpc/slicec/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/icerpc/slicec/actions?query=branch:main)

- [Build Requirements](#build-requirements)
- [Overview](#overview)
  - [Compile from strings](#compile-from-strings)
  - [Compile from options](#compile-from-options)
- [Testing](#testing)
- [Code Coverage Report](#code-coverage-report)

## Build Requirements

To build the slicec library you need to have Rust and Cargo installed. The recommended method to install Rust is by
using [rustup](https://rustup.rs).

## Overview

The slicec library is a Rust library that can be used to compile Slice definitions into a `CompilationResult` struct.
The `CompilationResult` struct contains the AST and any diagnostics that were emitted during compilation.

### Compile from strings

The simplest way to compile a Slice definition is by using the `compile_from_strings` function:

```rust
pub fn main() {
    let slice = "

    module GreeterExample

    /// Represents a simple greeter.
    interface Greeter {
        /// Creates a personalized greeting.
        /// @param name: The name of the person to greet.
        /// @returns: The greeting.
        greet(name: string) -> string
    }
    ";

    let compilation_result = slice::compile_from_strings(&[slice], None);
}
```

This function takes an array of strings containing Slice definitions and an optional set of compilation options.

### Compile from options

Alternatively, you can create `SliceOptions` and use the `compile_from_options` function to create a command line
application that compiles Slice definitions:

```rust
// main.rs
use slice::clap;
use slice::clap::Parser;
use slice::command_line::SliceOptions;

/// This struct is responsible for parsing the command line options.
/// The option parsing capabilities are generated on the struct by the `clap` macro.
#[derive(Debug, Parser)]
#[command(author, version, about, rename_all = "kebab-case")]
pub struct ExampleOptions {
    // Import the options common to all slice compilers.
    #[command(flatten)]
    pub slice_options: SliceOptions,
}

pub fn main() {
    let options = ExampleOptions::parse();
    let slice_options = &options.slice_options;
    let compilation_result = slice::compile_from_options(slice_options);
}
```

```slice
// greeter.slice

module GreeterExample

/// Represents a simple greeter.
interface Greeter {
    /// Creates a personalized greeting.
    /// @param name: The name of the person to greet.
    /// @returns: The greeting.
    greet(name: string) -> string
}
```

Build and run using Cargo:

```shell
cargo build
cargo run greeter.slice
```

## Testing

The test suite can be run from the command line by running `cargo test` in the repository.

## Code coverage report

Code coverage reports can be generated using [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) from a regular
command prompt, using the following command

For Linux and macOS:

```shell
./generate-coverage-report.sh
```

The output html is in the `target/llvm-cov/html/` directory.
