# IceRPC

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

Code coverage reports can be generated using [grcove](https://docs.rs/crate/grcov/0.4.3).

First install `grcov` and the `llvm-tools-preview`:

```shell
cargo install grcov
rustup component add llvm-tools-preview
```

Then set the relevant build flags:

```shell
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"
```

Finally generate the html report:

```shell
cargo +nightly build && cargo +nightly test
```

```shell
grcov . -s src --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/
```

The output html is in the `target/debug/coverage/` directory.
