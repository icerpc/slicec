#!/bin/bash

set -e

# This script compiles and checks all combinations of feature flags to ensure all of them are functional.
# It's intended as a temporary stop-gap until the crate is mature enough for something like CI to handle this instead.
# Yeah, I know, it's pretty lame. If it hurts you that much to look at, just stop looking at it.

# This line can be enabled and disabled to treat warnings as errors.
# export RUSTFLAGS=-Dwarnings

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

cargo test --no-default-features
cargo test --no-default-features --features alloc
cargo test --no-default-features --features std
cargo test --no-default-features --features bytes
cargo test --no-default-features --features std,bytes

echo

cargo test --no-default-features --features slice2
cargo test --no-default-features --features slice2,alloc
cargo test --no-default-features --features slice2,std
cargo test --no-default-features --features slice2,bytes
cargo test --no-default-features --features slice2,std,bytes

echo

cargo test --no-default-features --features slice1
cargo test --no-default-features --features slice1,alloc
cargo test --no-default-features --features slice1,std
cargo test --no-default-features --features slice1,bytes
cargo test --no-default-features --features slice1,std,bytes

echo

cargo test --no-default-features --features slice1,slice2
cargo test --no-default-features --features slice1,slice2,alloc
cargo test --no-default-features --features slice1,slice2,std
cargo test --no-default-features --features slice1,slice2,bytes
cargo test --no-default-features --features slice1,slice2,std,bytes

echo
echo
echo

cargo miri --no-default-features
cargo miri --no-default-features --features alloc
cargo miri --no-default-features --features std
cargo miri --no-default-features --features bytes
cargo miri --no-default-features --features std,bytes

echo

cargo miri --no-default-features --features slice2
cargo miri --no-default-features --features slice2,alloc
cargo miri --no-default-features --features slice2,std
cargo miri --no-default-features --features slice2,bytes
cargo miri --no-default-features --features slice2,std,bytes

echo

cargo miri --no-default-features --features slice1
cargo miri --no-default-features --features slice1,alloc
cargo miri --no-default-features --features slice1,std
cargo miri --no-default-features --features slice1,bytes
cargo miri --no-default-features --features slice1,std,bytes

echo

cargo miri --no-default-features --features slice1,slice2
cargo miri --no-default-features --features slice1,slice2,alloc
cargo miri --no-default-features --features slice1,slice2,std
cargo miri --no-default-features --features slice1,slice2,bytes
cargo miri --no-default-features --features slice1,slice2,std,bytes

echo
echo
echo

cargo doc --document-private-items --no-default-features --features slice1,slice2,std,bytes
