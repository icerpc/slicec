@echo off

REM This script compiles and checks all combinations of feature flags to ensure all of them are functional.
REM It's intended as a temporary stop-gap until the crate is mature enough for something like CI to handle this instead.
REM Yeah, I know, it's pretty lame. If it hurts you that much to look at, just stop looking at it.

REM This line can be enabled and disabled to treat warnings as errors.
REM SET RUSTFLAGS=-Dwarnings

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

cargo test --no-default-features
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features std
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo test --no-default-features --features slice2
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features slice2,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features slice2,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features slice2,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features slice2,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo test --no-default-features --features slice1
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features slice1,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features slice1,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features slice1,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features slice1,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo test --no-default-features --features slice1,slice2
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features slice1,slice2,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features slice1,slice2,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features slice1,slice2,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo test --no-default-features --features slice1,slice2,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.
ECHO.
ECHO.

cargo miri --no-default-features
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features std
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo miri --no-default-features --features slice2
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features slice2,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features slice2,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features slice2,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features slice2,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo miri --no-default-features --features slice1
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features slice1,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features slice1,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features slice1,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features slice1,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.

cargo miri --no-default-features --features slice1,slice2
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features slice1,slice2,alloc
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features slice1,slice2,std
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features slice1,slice2,bytes
if %ERRORLEVEL% NEQ 0 EXIT
cargo miri --no-default-features --features slice1,slice2,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT

ECHO.
ECHO.
ECHO.

cargo doc --document-private-items --no-default-features --features slice1,slice2,std,bytes
if %ERRORLEVEL% NEQ 0 EXIT
