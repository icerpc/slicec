<p align="center">
  <img src="https://github.com/icerpc/slicec/raw/main/.github/assets/slicec-banner.svg" height="100" width="100" />
</p>

# The Slice compiler (slicec)

[![CI](https://github.com/icerpc/slicec/actions/workflows/ci.yml/badge.svg)][ci-home]
[![License](https://img.shields.io/github/license/icerpc/slicec?color=blue)][license]

To build slicec you must have Rust and Cargo installed.
To install these, we recommend reading the following [guide](https://doc.rust-lang.org/cargo/getting-started/installation.html).

### Building

Run the following command to build slicec and its dependencies:
```shell
cargo build
```

### Running the tests

Run the following command to run the test suite:
```shell
cargo test
```

### Generating documentation

To generate documentation for slicec, run the following command:
```shell
cargo doc --no-deps --document-private-items
```
This will generate documentation in `target/doc/slicec/`.
However, you can easily view the documentation after generating it with:
```shell
cargo doc --no-deps --document-private-items --open
```

### Generating a code coverage report

slicec uses the [llvm-cov](https://crates.io/crates/cargo-llvm-cov) Cargo subcommand to generate coverage reports.
So, to generate reports you must install it by running:
```shell
cargo install cargo-llvm-cov
```

To generate a coverage report for slicec, run the following command:
```shell
cargo llvm-cov --html
```
This will generate an HTML report in `target/llvm-cov/html/`.
However, you can easily view the report after generating it with:
```shell
cargo llvm-cov --open
```

[ci-home]: https://github.com/icerpc/slicec/actions/workflows/ci.yml
[license]: https://github.com/icerpc/slicec/blob/main/LICENSE
[slice]: https://docs.icerpc.dev/slice2
