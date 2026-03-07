# slicec

To build slice-rust you must have Rust and Cargo installed.
To install these, we recommend reading the following [guide](https://doc.rust-lang.org/cargo/getting-started/installation.html).

## Building

Run the following command to build slice-rust and its dependencies:

```shell
cargo build
```

## Running the tests

Run the following command to run the test suite:

```shell
cargo test
```

## Generating documentation

To generate documentation for slice-rust, run the following command:

```shell
cargo doc --no-deps --document-private-items
```

This will generate documentation in the `target/doc/` directory.

Additionally, you can easily view the documentation after generating it with the `open` flag:

```shell
cargo doc --no-deps --document-private-items --open
```
