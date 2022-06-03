#!/usr/bin/env bash

# Ensure that necessary depencies are installed:
FILE=~/.cargo/bin/grcov
if [ ! -f "$FILE" ]; then
    echo "grov is missing"
    echo "run 'cargo install grov' to install"
    exit 1
fi

if ! rustup component list | grep llvm-tools-preview > /dev/null; then
    echo "llvm-tools-preview is not installed"
    echo  "run 'rustup component add llvm-tools-preview' to install"
    exit 1
fi

# Set the relevant build flags:
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"

# Generate the html report:
cargo +nightly build && cargo +nightly test
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/

FILE=target/debug/coverage/index.html
if [ -f "$FILE" ]; then
    echo ""
    echo "Successfully generated the coverage report"
    echo "The report can be found at 'target/debug/coverage'"
else
    echo ""
    echo "Failed to generate the coverage report"
    echo "Please check the above output for any errors"
fi

# Cleanup build flags:
unset CARGO_INCREMENTAL
unset RUSTFLAGS
unset RUSTDOCFLAGS
