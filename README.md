# Slice compiler library (slicec)

[![.github/workflows/ci.yml](https://github.com/icerpc/slicec/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/icerpc/slicec/actions?query=branch:main)

The slicec library is a Rust library that compiles [Slice][slice] definitions into a `CompilationState` struct. The
`CompilationState` struct contains the AST and diagnostics emitted during compilation (if any).

## Build requirements

Install Rust and Cargo using [rustup](https://rustup.rs/).

## Compile from strings

The simplest way to compile a Slice definition is by using the `compile_from_strings` function:

```rust
pub fn main() {
    let slice = "

    module VisitorCenter

    /// Represents a simple greeter.
    interface Greeter {
        /// Creates a personalized greeting.
        /// @param name: The name of the person to greet.
        /// @returns: The greeting.
        greet(name: string) -> string
    }
    ";

    let compilation_state = slice::compile_from_strings(&[slice], None);
}
```

This function takes an array of strings containing Slice definitions and an optional set of compilation options.

## Compile from options

Alternatively, you can create `SliceOptions` and use the `compile_from_options` function to create a command line
application that compiles Slice definitions:

```rust
// main.rs
pub fn main() {
    let options = SliceOptions::parse();
    let slice_options = &options.slice_options;
    let compilation_state = slice::compile_from_options(slice_options);
}
```

```slice
// greeter.slice

module VisitorCenter

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
cargo install cargo-llvm-cov
cargo llvm-cov --html
```

The output html is in the `target/llvm-cov/html/` directory.

[slice]: https://docs.testing.zeroc.com/slice2
