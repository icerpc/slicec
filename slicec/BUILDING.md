# Building from source

Run the following command to build `slicec` (the library and the binary) and its dependencies:

```shell
cargo build
```

## Table of contents

- [Building from source](#building-from-source)
  - [Table of contents](#table-of-contents)
  - [Prerequisites](#prerequisites)
  - [Running the tests](#running-the-tests)
  - [Generating documentation](#generating-documentation)
  - [Generating a code coverage report](#generating-a-code-coverage-report)

## Prerequisites

To build slicec you must have Rust and Cargo installed.
To install these, we recommend reading the following [guide](https://doc.rust-lang.org/cargo/getting-started/installation.html).

## Running the tests

Run the following command to run the test suite:

```shell
cargo test
```

## Generating documentation

To generate documentation for slicec, run the following command:

```shell
cargo doc --no-deps --document-private-items
```
This will generate documentation in the `target/doc/slicec` directory.

Additionally, you can easily view the documentation after generating it with the `open` flag:

```shell
cargo doc --no-deps --document-private-items --open
```

## Generating a code coverage report

slicec uses [llvm-cov](https://crates.io/crates/cargo-llvm-cov) to generate coverage reports.
So, to generate reports you must install it:

```shell
cargo install cargo-llvm-cov
```

To generate a coverage report for slicec, run the following command:

```shell
cargo llvm-cov --html
```
This will generate an HTML report in the `target/llvm-cov/html` directory.

Additionally, you can easily view the report after generating it with the `open` flag:

```shell
cargo llvm-cov --open
```
