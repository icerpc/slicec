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


```shell
cargo install rustfilt
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

```shell
RUSTFLAGS="-C instrument-coverage" \
    LLVM_PROFILE_FILE="tests/test-report/data/test-report-%m.profraw" \
    cargo test --tests
```

```shell
cargo profdata -- merge -sparse tests/test-report/data/test-report-*.profraw -o tests/test-report/data/test-report.profdata
```

```shell
cargo cov -- report \
    $( \
      for file in \
        $( \
          RUSTFLAGS="-C instrument-coverage" \
            cargo test --tests --no-run --message-format=json \
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
        ); \
      do \
        printf "%s %s " -object $file; \
      done \
    ) \
  --instr-profile=tests/test-report/data/test-report.profdata --summary-only \
  --use-color --ignore-filename-regex='/.cargo/registry'
```