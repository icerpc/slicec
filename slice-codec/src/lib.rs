// Copyright (c) ZeroC, Inc.

//! TODO write a doc comment explaining this crate.

// If the 'alloc' feature is set, pull in [`alloc`](https://doc.rust-lang.org/alloc) as an external crate.
#[cfg(feature = "alloc")]
extern crate alloc;

// If the 'std' feature is set, pull in [`std`](https://doc.rust-lang.org/std) as an external crate.
// Note that the [`prelude`](https://doc.rust-lang.org/std/prelude) is still disabled (for safety),
// so we need to fully qualify (or explicitly `use`) types that normally would be pulled in automatically.
#[cfg(feature = "std")]
extern crate std;

// Only include the `slice2` module if the corresponding feature is set.
#[cfg(feature = "slice2")]
pub mod slice2;

// Only include the `slice1` module if the corresponding feature is set.
#[cfg(feature = "slice1")]
pub mod slice1;

pub mod decoder;
pub mod decoding;
pub mod encoder;
pub mod encoding;
pub mod io;
pub mod try_decode;
pub mod try_encode;

/// A specialized [`Result`](core::result::Result) type for decoding and encoding functions which may produce errors.
///
/// This typedef is a convenience to avoid repetitively specifying [`Error`] as the error type, and is otherwise a
/// direct mapping to  [`core::result::Result`].
pub type Result<T> = core::result::Result<T, Error>;

/// The error type for encoding and decoding functions.
pub enum Error {}
