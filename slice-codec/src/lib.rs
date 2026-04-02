// Copyright (c) ZeroC, Inc.

//! TODO write a doc comment explaining this crate.

#![no_std]

// If the 'alloc' feature is set, pull in [`alloc`](https://doc.rust-lang.org/alloc) as an external crate.
#[cfg(feature = "alloc")]
extern crate alloc;

// If the 'std' feature is set, pull in [`std`](https://doc.rust-lang.org/std) as an external crate.
// Even with this feature, parts of the [`prelude`](https://doc.rust-lang.org/std/prelude) are still disabled,
// so we need to fully qualify (or explicitly `use`) some types that would normally be pulled in automatically.
#[cfg(feature = "std")]
extern crate std;

// These modules are private because they don't export any types, just implementations.
mod decoding;
mod encoding;

pub mod buffer;
pub mod decode_from;
pub mod decoder;
pub mod encode_into;
pub mod encoder;

// Re-export the contents of the `error` module directly into the crate root, so they're easier to reference.
mod error;
pub use error::*;

/// The smallest value that can be represented as a `varint32`.
pub const VARINT32_MIN: i32 = i32::MIN;
/// The largest value that can be represented as a `varint32`.
pub const VARINT32_MAX: i32 = i32::MAX;
/// The smallest value that can be represented as a `varuint32`.
pub const VARUINT32_MIN: u32 = u32::MIN;
/// The largest value that can be represented as a `varuint32`.
pub const VARUINT32_MAX: u32 = u32::MAX;
/// The smallest value that can be represented as a `varint62`.
pub const VARINT62_MIN: i64 = i64::MIN >> 2;
/// The largest value that can be represented as a `varint62`.
pub const VARINT62_MAX: i64 = i64::MAX >> 2;
/// The smallest value that can be represented as a `varuint62`.
pub const VARUINT62_MIN: u64 = u64::MIN >> 2;
/// The largest value that can be represented as a `varuint62`.
pub const VARUINT62_MAX: u64 = u64::MAX >> 2;
