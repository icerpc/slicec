#!/usr/bin/env bash

set -e

# Ensure that necessary dependencies are installed.
if ! cargo llvm-cov -V &> /dev/null; then
    echo "cargo-llvm-cov is missing"
    echo "run 'cargo install cargo-llvm-cov' to install"
    exit 1
fi

# Generate the html report.
cargo llvm-cov --html

FILE=target/llvm-cov/html/index.html
if [ -f "$FILE" ]; then
    echo ""
    echo "Successfully generated the coverage report"
    echo "The report can be found at 'target/llvm-cov/html'"
else
    echo ""
    echo "Failed to generate the coverage report"
    echo "Please check the above output for any errors"
fi
