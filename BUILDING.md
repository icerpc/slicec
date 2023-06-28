
## Prerequisites

The slicec library (and the compilers that use it) are written in Rust.
So you must have Rust installed: [rustup](https://rustup.rs).

## Building and Testing

To build:
```
cargo build
```

To run the test suite:
```
cargo test
```

## Generating Documentation

To generate the crate's documentation:
```
cargo doc --no-deps --document-private-items
```
By default this will generate the documentation in `target/doc/slicec`.
However, you can easily view the documentation after generating it with:
```
cargo doc --no-deps --document-private-items --open
```

## Generating a Code Coverage Report

slicec uses the [llvm-cov](https://crates.io/crates/cargo-llvm-cov) Cargo subcommand to generate coverage reports.
So, to generate reports you must install it:
```
cargo install cargo-llvm-cov
```

To generate the crate's coverage report:
```
cargo llvm-cov --html
```
By default this will generate the report in `target/llvm-cov/html`.
However, you can easily view the report after generating it with:
```
cargo llvm-cov --open
```
