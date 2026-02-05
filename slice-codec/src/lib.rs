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

// Only include the `slice2` module if the corresponding feature is set.
#[cfg(feature = "slice2")]
pub mod slice2;

pub mod buffer;
pub mod decode_from;
pub mod decoder;
pub mod encode_into;
pub mod encoder;

// Re-export the contents of the `error` module directly into the crate root, so they're easier to reference.
mod error;
pub use error::*;

/// A marker for types which represent a specific version/implementation of a Slice encoding.
///
/// Types which implement this trait can be used to specialize implementations of the
/// [`DecodeFrom`](decode_from::DecodeFrom) and [`EncodeInto`](encode_into::EncodeInto) traits,
/// and as a type argument for [`Encoders`](encoder::Encoder) and [`Decoders`](decoder::Decoder).
pub trait Encoding: Sized {}
