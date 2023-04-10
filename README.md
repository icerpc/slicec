# Slice Compiler Library (slicec)

[![.github/workflows/rust.yml](https://github.com/icerpc/icerpc/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/icerpc/icerpc/actions?query=branch:main)

- [Build Requirements](#build-requirements)
- [Usage](#usage)
- [Testing](#testing)
- [Code Coverage Report](#code-coverage-report)

## Build Requirements

You'll need Rust and Cargo to run and build the compiler. I'd personally recommend rustup:
<https://rustup.rs>

## Usage

You can run the compiler directly with cargo (assuming you're running from the base directory):

```shell
cargo run example/testing.slice
```

Or build the slice library and the slicec-csharp binary, and use them directly:

```shell
cargo build --release
target/release/slicec-csharp example/testing.slice
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
