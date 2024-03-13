@echo off

REM This script compiles and checks all combinations of feature flags to ensure all of them are functional.
REM It's intended as a temporary stop-gap until the crate is mature enough for something like CI to handle this instead.
REM Yeah, I know, it's pretty lame. If it hurts you that much to look at, just stop looking at it.

REM This line can be enabled and disabled to treat warnings as errors.
REM SET RUSTFLAGS=-Dwarnings

REM Build the crate with each combination of features.

cargo build --no-default-features
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features std
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo build --no-default-features --features slice2
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features slice2,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features slice2,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features slice2,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features slice2,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo build --no-default-features --features slice1
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features slice1,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features slice1,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features slice1,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features slice1,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo build --no-default-features --features slice1,slice2
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features slice1,slice2,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features slice1,slice2,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features slice1,slice2,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo build --no-default-features --features slice1,slice2,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.
ECHO.
ECHO.

REM Lint the crate with each combination of features.

cargo clippy --all-targets --no-default-features
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features std
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo clippy --all-targets --no-default-features --features slice2
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features slice2,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features slice2,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features slice2,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features slice2,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo clippy --all-targets --no-default-features --features slice1
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features slice1,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features slice1,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features slice1,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features slice1,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo clippy --all-targets --no-default-features --features slice1,slice2
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features slice1,slice2,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features slice1,slice2,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features slice1,slice2,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo clippy --all-targets --no-default-features --features slice1,slice2,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.
ECHO.
ECHO.

REM We use miri to run the tests, to catch memory issues.
REM We always set the 'slice1' and 'slice2' features to save time testing, and because these tests are already isolated.

cargo +nightly miri test --no-default-features --features slice1,slice2
if %ERRORLEVEL% NEQ 0 EXIT
cargo +nightly miri test --no-default-features --features slice1,slice2,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo +nightly miri test --no-default-features --features slice1,slice2,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo +nightly miri test --no-default-features --features slice1,slice2,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo +nightly miri test --no-default-features --features slice1,slice2,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.
ECHO.
ECHO.

REM Generate the docs with each combination of features to ensure we aren't incorrectly linking to feature gated things.

cargo doc --document-private-items --no-default-features
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features std
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo doc --document-private-items --no-default-features --features slice2
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features slice2,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features slice2,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features slice2,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features slice2,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo doc --document-private-items --no-default-features --features slice1
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features slice1,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features slice1,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features slice1,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features slice1,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo doc --document-private-items --no-default-features --features slice1,slice2
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features slice1,slice2,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features slice1,slice2,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features slice1,slice2,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo doc --document-private-items --no-default-features --features slice1,slice2,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT
