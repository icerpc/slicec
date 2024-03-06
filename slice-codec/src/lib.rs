// Copyright (c) ZeroC, Inc.

//! TODO write a big doc comment explaining this crate.

#![no_std]

// If the 'std' feature is set, pull in `std` as an external crate.
// Note that the prelude is still disabled, so you'll need to use explicit paths for types provided by `std`.
#[cfg(feature = "std")]
extern crate std;

// If the 'alloc' feature is set, pull in `alloc` as an external crate.
#[cfg(feature = "alloc")]
extern crate alloc;

// Only include the `slice2` module if the corresponding feature is set.
#[cfg(feature = "slice2")]
pub mod slice2;

// Only include the `slice1` module if the corresponding feature is set.
#[cfg(feature = "slice1")]
pub mod slice1;

pub mod buf_io;
pub mod encoder;
pub mod decoder;
pub mod try_encode;
pub mod try_decode;

// These modules are private because they don't export any types, just implementations.
mod encoding;
mod decoding;

use encoder::Encoder;
use decoder::Decoder;

use core::fmt::Debug;

/// TODO
pub trait Encoding: Clone + Copy + Debug + Default + PartialEq + Eq {
    /// TODO
    fn try_encode_size(size: usize, encoder: &mut Encoder<Self>) -> Result<()>;

    /// TODO
    fn try_decode_size(decoder: &mut Decoder<Self>) -> Result<usize>;
}

/// A specialized `Result` type for encoding and decoding functions which may produce an error.
///
/// It is a direct mapping to [`std::result::Result`] with an `Err` type of [`Error`].
pub type Result<T> = core::result::Result<T, Error>;

/// TODO
#[derive(Debug)]
pub enum Error {
    /// TODO
    InvalidData {
        /// TODO
        desc: &'static str,
    },

    /// TODO
    IllegalValue {
        /// TODO
        value: i128,
        /// TODO
        desc: &'static str,
    },

    /// TODO
    OutOfRange {
        /// TODO
        value: i128,
        /// TODO
        min: i128,
        /// TODO
        max: i128,
        /// TODO
        typename: &'static str,
    },

    /// TODO
    HeapAllocationLimitReached {
        /// TODO
        limit: usize,
        /// TODO
        current: usize,
        /// TODO
        requested: usize,
    },

    /// TODO
    EndOfBuffer {
        /// TODO
        attempted: usize,
        /// TODO
        remaining: usize,
    },
}
