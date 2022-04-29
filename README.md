# IceRpc

This repository contains the source code for the IceRpc project.

## Requirements

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

## Examples

There are 3 example files, one with no errors that generates code:

```shell
target/release/slicec-csharp example/testing.slice
```

one that showcases a syntax error, when the parser can't parse the input file and dies:

```shell
target/release/slicec-csharp example/syntaxerrors.slice
```

and another which shows a semantic error, where the input can be parsed, but there's an error in the user's definitions:

```shell
target/release/slicec-csharp example/sliceerrors.slice
```

The compiler doesn't support passing directores, but does support multiple files being passed in. For instance, running:

```shell
target/release/slicec-csharp example/testing.slice example/sliceerrors.slice
```

will show a redefinition conflict between the 2 slice files.

## Testing

The test suite can be run from the command line by running `cargo test` command in the repository top-level
directory.

Code coverage reports can be generated using [grcove](https://docs.rs/crate/grcov/0.4.3).

First we must  install grcov and the llvm tools

```shell
cargo install grcov
rustup component add llvm-tools-preview
```

Then set the flags required for generating the .profraw files.

```shell
export CARGO_INCREMENTAL=0 \
&& export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort" \
&&  export RUSTDOCFLAGS="-Cpanic=abort"
```

Finally generate the .profraw files.

```shell
cargo build && cargo test
```

Now that the files are generate  grcov can generate the html report:

```shell
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/
```

The output html is in the `target/debug/coverage/` directory.
