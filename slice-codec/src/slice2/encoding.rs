// Copyright (c) ZeroC, Inc.

use super::*;
use crate::buffer::OutputTarget;
use crate::encode_into::*;
use crate::encoder::Encoder;
use crate::{Error, InvalidDataErrorKind, Result};

// We only support `BTreeMap` if the `alloc` crate is available through the `alloc` feature flag.
#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

// We only support `HashMap` if the standard library is available through the `std` feature flag.
#[cfg(feature = "std")]
use std::collections::HashMap;

// =============================================================================
// Fixed-length type implementations
// =============================================================================

impl EncodeInto<Slice2> for bool {
    /// Encodes a value of `0` for `false` or a value of `1` for true, on a single byte.
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget, Slice2>) -> Result<()> {
        // In memory, bools are guaranteed to be `0` for `false` and `1` for `true`.
        encoder.write_byte(self as u8)
    }
}
implement_encode_into_on_borrowed_type!(bool, Slice2);

impl EncodeInto<Slice2> for u8 {
    /// Writes this byte directly to the buffer, as is.
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget, Slice2>) -> Result<()> {
        encoder.write_byte(self)
    }
}
implement_encode_into_on_borrowed_type!(u8, Slice2);

impl EncodeInto<Slice2> for i8 {
    /// Writes this `i8` directly to the buffer, treating it as-if it was a `u8`.
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget, Slice2>) -> Result<()> {
        // Casting between two integers of the same size (`u8` and `i8`) is no-op in Rust.
        encoder.write_byte(self as u8)
    }
}
implement_encode_into_on_borrowed_type!(i8, Slice2);

implement_encode_into_on_numeric_primitive_type! {u16, Slice2, "Encodes this [`u16`] on 2 bytes (little endian)."}
implement_encode_into_on_numeric_primitive_type! {i16, Slice2, "Encodes this [`i16`] on 2 bytes (little endian) in two's complement form."}
implement_encode_into_on_numeric_primitive_type! {u32, Slice2, "Encodes this [`u32`] on 4 bytes (little endian)."}
implement_encode_into_on_numeric_primitive_type! {i32, Slice2, "Encodes this [`i32`] on 4 bytes (little endian) in two's complement form."}
implement_encode_into_on_numeric_primitive_type! {u64, Slice2, "Encodes this [`u64`] on 8 bytes (little endian)."}
implement_encode_into_on_numeric_primitive_type! {i64, Slice2, "Encodes this [`i64`] on 8 bytes (little endian) in two's complement form."}
implement_encode_into_on_numeric_primitive_type! {f32, Slice2, "Encodes this [`f32`] on 4 bytes (little endian) using the \"binary32\" representation defined in IEEE 754-2008."}
implement_encode_into_on_numeric_primitive_type! {f64, Slice2, "Encodes this [`f64`] on 8 bytes (little endian) using the \"binary64\" representation defined in IEEE 754-2008."}

// =============================================================================
// Variable-length integer type implementations
// =============================================================================

/// TODO
fn varint_range_error(value: impl Into<i128>) -> Error {
    let error = InvalidDataErrorKind::OutOfRange {
        value: value.into(),
        min: VARINT62_MIN as i128,
        max: VARINT62_MAX as i128,
        typename: "varint62",
    };
    error.into()
}

/// TODO
fn varuint_range_error(value: impl Into<i128>) -> Error {
    let error = InvalidDataErrorKind::OutOfRange {
        value: value.into(),
        min: VARUINT62_MIN as i128,
        max: VARUINT62_MAX as i128,
        typename: "varuint62",
    };
    error.into()
}

impl<O: OutputTarget> Encoder<O, Slice2> {
    /// Encodes a signed integer on between 1 and 8 bytes in the variable length '[varint]' format.
    ///
    /// [varint]: https://docs.icerpc.dev/slice2/language-guide/primitive-types#variable-size-integral-types
    #[rustfmt::skip] // To keep the arms of `match required_bits` aligned for readability.
    pub fn encode_varint(&mut self, value: impl Into<i64>) -> Result<()> {
        let value: i64 = value.into();

        // See: https://docs.icerpc.dev/slice2/encoding/primitive-types#variable-size-integral-types.

        // Compute how many bits are required to encode this value.
        let mut required_bits = i64::BITS - match value.is_negative() {
            // If the value is non-negative, we can ignore any leading `0` bits.
            false => value.leading_zeros(),
            // If the value is negative, we can ignore any leading `1` bits.
            true => value.leading_ones(),
        };
        required_bits += 1; // Add '1' to account for the sign bit.
        // We have to shift the value up by 2 bits to make room for the size prefix.
        let shifted_value: i64 = value << 2;

        match required_bits {
            0..=6   => self.encode(shifted_value as i8),
            7..=14  => self.encode(shifted_value as i16 | 0b01),
            15..=30 => self.encode(shifted_value as i32 | 0b10),
            31..=62 => self.encode(shifted_value | 0b11),
            63.. => Err(varint_range_error(value)),
        }
    }

    /// Encodes an unsigned integer on between 1 and 8 bytes in the variable length '[varuint]' format.
    ///
    /// [varuint]: https://docs.icerpc.dev/slice2/language-guide/primitive-types#variable-size-integral-types
    #[rustfmt::skip] // To keep the arms of `match required_bits` aligned for readability.
    pub fn encode_varuint(&mut self, value: impl Into<u64>) -> Result<()> {
        let value: u64 = value.into();

        // See: https://docs.icerpc.dev/slice2/encoding/primitive-types#variable-size-integral-types.

        // Compute how many bits are required to encode this value.
        let required_bits = u64::BITS - value.leading_zeros();
        // We have to shift the value up by 2 bits to make room for the size prefix.
        let shifted_value: u64 = value << 2;

        match required_bits {
            0..=6   => self.encode(shifted_value as u8),
            7..=14  => self.encode(shifted_value as u16 | 0b01),
            15..=30 => self.encode(shifted_value as u32 | 0b10),
            31..=62 => self.encode(shifted_value | 0b11),
            63.. => Err(varuint_range_error(value)),
        }
    }

    // An alias for `[encode_varint]` to increase readability.
    pub fn encode_size(&mut self, value: impl Into<u64>) -> Result<()> {
        self.encode_varuint(value)
    }
}

// =============================================================================
// Sequence type implementations
// =============================================================================

/// TODO
impl EncodeInto<Slice2> for &str {
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget, Slice2>) -> Result<()> {
        let size = u64::try_from(self.len())?;
        encoder.encode_size(size)?;
        encoder.write_bytes_exact(self.as_bytes())
    }
}

/// TODO
impl<'a, T> EncodeInto<Slice2> for &'a [T]
where
    &'a T: EncodeInto<Slice2>,
{
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget, Slice2>) -> Result<()> {
        let size = u64::try_from(self.len())?;
        encoder.encode_size(size)?;
        for element in self {
            encoder.encode(element)?;
        }
        Ok(())
    }
}

// =============================================================================
// Dictionary type implementations
// =============================================================================

#[cfg(feature = "alloc")]
impl_encode_into_on_dictionary_type!(BTreeMap, Slice2, "TODO");

#[cfg(feature = "std")]
impl_encode_into_on_dictionary_type!(HashMap, Slice2, "TODO");

// TODO add support for optional sequences and dictionaries
