// Copyright (c) ZeroC, Inc.

// These modules are private because they don't export any types, just implementations.
mod decoding;
mod encoding;

use crate::decoder::Decoder;
use crate::encoder::Encoder;
use crate::{Encoding, Error, Result};

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

/// Convenience definition for an [`Encoder`] using the [`Slice2`] encoding.
pub type Slice2Encoder<'a> = Encoder<'a, Slice2>;
/// Convenience definition for a [`Decoder`] using the [`Slice2`] encoding.
pub type Slice2Decoder<'a> = Decoder<'a, Slice2>;

/// Version 2 of the Slice encoding.
/// This is the default version used by this crate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Slice2;

impl Encoding for Slice2 {
    /// The Slice2 encoding uses `varuint62` for encoding sizes and lengths.
    /// So, this function decodes a `varuint62` and returns it as a [`usize`].
    fn try_decode_size(decoder: &mut Slice2Decoder) -> Result<usize> {
        // Try to decode a `varuint62` as a `u64`.
        let varuint62 = decoder.try_decode_varuint62()?;

        // Try to convert the `u64` into a `usize`, and return an error if it's too large.
        varuint62.try_into().map_err(|_| Error::OutOfRange {
            value: varuint62 as i128,
            min: usize::MIN as i128,
            max: usize::MAX as i128,
            typename: "usize",
        })
    }

    /// The Slice2 encoding uses `varuint62` for encoding sizes and lengths.
    /// So, this function takes a [`usize`] and encodes it as a `varuint62`.
    fn try_encode_size(size: usize, encoder: &mut Slice2Encoder) -> Result<()> {
        // Try to convert the `usize` into a `u64`, and return an error if it's too large.
        let varuint62 = size.try_into().map_err(|_| Error::OutOfRange {
            value: size as i128,
            min: VARUINT62_MIN as i128,
            max: VARUINT62_MAX as i128,
            typename: "varuint62",
        })?;

        // TODO
        // Try to encode the `u64` as a `varuint62`.
        let temp: u64 = varuint62;
        todo!() //TODO UNCOMMENT MEencoder.try_encode_varuint62(varuint62)
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
