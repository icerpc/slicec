#!/bin/bash

set -e

# This script compiles and checks all combinations of feature flags to ensure all of them are functional.
# It's intended as a temporary stop-gap until the crate is mature enough for something like CI to handle this instead.
# Yeah, I know, it's pretty lame. If it hurts you that much to look at, just stop looking at it.

# This line can be enabled and disabled to treat warnings as errors.
# export RUSTFLAGS=-Dwarnings

# Build the crate with each combination of features.

cargo build --no-default-features
cargo build --no-default-features --features alloc
cargo build --no-default-features --features std
cargo build --no-default-features --features bytes
cargo build --no-default-features --features std,bytes

echo

cargo build --no-default-features --features slice2
cargo build --no-default-features --features slice2,alloc
cargo build --no-default-features --features slice2,std
cargo build --no-default-features --features slice2,bytes
cargo build --no-default-features --features slice2,std,bytes

echo

cargo build --no-default-features --features slice1
cargo build --no-default-features --features slice1,alloc
cargo build --no-default-features --features slice1,std
cargo build --no-default-features --features slice1,bytes
cargo build --no-default-features --features slice1,std,bytes

echo

cargo build --no-default-features --features slice1,slice2
cargo build --no-default-features --features slice1,slice2,alloc
cargo build --no-default-features --features slice1,slice2,std
cargo build --no-default-features --features slice1,slice2,bytes
cargo build --no-default-features --features slice1,slice2,std,bytes

echo
echo
echo

# Lint the crate with each combination of features.

cargo clippy --all-targets --no-default-features
cargo clippy --all-targets --no-default-features --features alloc
cargo clippy --all-targets --no-default-features --features std
cargo clippy --all-targets --no-default-features --features bytes
cargo clippy --all-targets --no-default-features --features std,bytes

echo

cargo clippy --all-targets --no-default-features --features slice2
cargo clippy --all-targets --no-default-features --features slice2,alloc
cargo clippy --all-targets --no-default-features --features slice2,std
cargo clippy --all-targets --no-default-features --features slice2,bytes
cargo clippy --all-targets --no-default-features --features slice2,std,bytes

echo

cargo clippy --all-targets --no-default-features --features slice1
cargo clippy --all-targets --no-default-features --features slice1,alloc
cargo clippy --all-targets --no-default-features --features slice1,std
cargo clippy --all-targets --no-default-features --features slice1,bytes
cargo clippy --all-targets --no-default-features --features slice1,std,bytes

echo

cargo clippy --all-targets --no-default-features --features slice1,slice2
cargo clippy --all-targets --no-default-features --features slice1,slice2,alloc
cargo clippy --all-targets --no-default-features --features slice1,slice2,std
cargo clippy --all-targets --no-default-features --features slice1,slice2,bytes
cargo clippy --all-targets --no-default-features --features slice1,slice2,std,bytes

echo
echo
echo

# We use miri to run the tests, to catch memory issues.
# We always set the 'slice1' and 'slice2' features to save time testing, and because these tests are already isolated.

cargo +nightly miri test --no-default-features --features slice1,slice2
cargo +nightly miri test --no-default-features --features slice1,slice2,alloc
cargo +nightly miri test --no-default-features --features slice1,slice2,std
cargo +nightly miri test --no-default-features --features slice1,slice2,bytes
cargo +nightly miri test --no-default-features --features slice1,slice2,std,bytes

echo
echo
echo

# Generate the docs with each combination of features to ensure we aren't incorrectly linking to feature gated things.

cargo doc --document-private-items --no-default-features
cargo doc --document-private-items --no-default-features --features alloc
cargo doc --document-private-items --no-default-features --features std
cargo doc --document-private-items --no-default-features --features bytes
cargo doc --document-private-items --no-default-features --features std,bytes

echo

cargo doc --document-private-items --no-default-features --features slice2
cargo doc --document-private-items --no-default-features --features slice2,alloc
cargo doc --document-private-items --no-default-features --features slice2,std
cargo doc --document-private-items --no-default-features --features slice2,bytes
cargo doc --document-private-items --no-default-features --features slice2,std,bytes

echo

cargo doc --document-private-items --no-default-features --features slice1
cargo doc --document-private-items --no-default-features --features slice1,alloc
cargo doc --document-private-items --no-default-features --features slice1,std
cargo doc --document-private-items --no-default-features --features slice1,bytes
cargo doc --document-private-items --no-default-features --features slice1,std,bytes

echo

cargo doc --document-private-items --no-default-features --features slice1,slice2
cargo doc --document-private-items --no-default-features --features slice1,slice2,alloc
cargo doc --document-private-items --no-default-features --features slice1,slice2,std
cargo doc --document-private-items --no-default-features --features slice1,slice2,bytes
cargo doc --document-private-items --no-default-features --features slice1,slice2,std,bytes
