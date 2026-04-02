// Copyright (c) ZeroC, Inc.

use crate::buffer::InputSource;
use crate::decode_from::*;
use crate::decoder::Decoder;
use crate::{Error, InvalidDataErrorKind, Result};

// We only support `String`, `Vec`, and `BTreeMap` if the `alloc` crate is available through the `alloc` feature flag.
#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

// We only support `HashMap` if the standard library is available through the `std` feature flag.
#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::hash::Hash;

/// TAG_END_MARKER must be encoded at the end of every non-compact type.
const TAG_END_MARKER: i32 = -1;

// =============================================================================
// Fixed-length type implementations
// =============================================================================

/// TODO
fn illegal_bool_error(value: u8) -> Error {
    let error = InvalidDataErrorKind::IllegalValue {
        desc: "bools can only have a numeric value of either '0' or '1'",
        value: Some(value as i128),
    };
    error.into()
}

impl DecodeFrom for bool {
    /// Reads a single byte from the buffer and returns `false` if it is `0` or `true` if it is `1`.
    /// If the byte has any other value, an error is returned instead.
    fn decode_from(decoder: &mut Decoder<impl InputSource>) -> crate::Result<Self> {
        let byte = decoder.read_byte()?;

        // We strictly enforce the Slice spec; A bool _must_ be encoded as either `0` or `1`.
        match byte {
            0 | 1 => Ok(byte != 0),
            _ => Err(illegal_bool_error(byte)),
        }
    }
}

impl DecodeFrom for u8 {
    /// Reads a single byte directly from the buffer and returns it, as is.
    fn decode_from(decoder: &mut Decoder<impl InputSource>) -> crate::Result<Self> {
        decoder.read_byte()
    }
}

impl DecodeFrom for i8 {
    /// Reads a single byte directly from the buffer and returns it, as-if it was an `i8`.
    fn decode_from(decoder: &mut Decoder<impl InputSource>) -> crate::Result<Self> {
        // In Rust, signed-integers are guaranteed to use a two's complement representation in memory.
        // Casting between `u8` and `i8` is no-op, and doesn't change this representation, or the sign bit.
        let byte = decoder.read_byte()?;
        Ok(byte as i8)
    }
}

implement_decode_from_on_numeric_primitive_type! {u16, "Decodes a [`u16`] from 2 bytes (little endian)."}
implement_decode_from_on_numeric_primitive_type! {i16, "Decodes a [`i16`] from 2 bytes (little endian) in two's complement form."}
implement_decode_from_on_numeric_primitive_type! {u32, "Decodes a [`u32`] from 4 bytes (little endian)."}
implement_decode_from_on_numeric_primitive_type! {i32, "Decodes a [`i32`] from 4 bytes (little endian) in two's complement form."}
implement_decode_from_on_numeric_primitive_type! {u64, "Decodes a [`u64`] from 8 bytes (little endian)."}
implement_decode_from_on_numeric_primitive_type! {i64, "Decodes a [`i64`] from 8 bytes (little endian) in two's complement form."}
implement_decode_from_on_numeric_primitive_type! {f32, "Decodes a [`f32`] from 4 bytes (little endian) using the \"binary32\" representation defined in IEEE 754-2008."}
implement_decode_from_on_numeric_primitive_type! {f64, "Decodes a [`f64`] from 8 bytes (little endian) using the \"binary64\" representation defined in IEEE 754-2008."}

// =============================================================================
// Variable-length integer type implementations
// =============================================================================

/// TODO
// TODO this isn't great. It assumes `T` is a signed integer, and has a size less than `u32::MAX`. For sane users,
// these will always be true. But if these assumptions don't hold, the 'min' and 'max' this reports will be wrong.
fn varint_range_error<T>(value: i64) -> Error {
    let size = core::mem::size_of::<T>() as u32;
    let shift_count = i128::BITS - (size * 8);
    let error = InvalidDataErrorKind::OutOfRange {
        value: value as i128,
        min: i128::MIN >> shift_count,
        max: i128::MAX >> shift_count,
        typename: core::any::type_name::<T>(),
    };
    error.into()
}

/// TODO
// TODO this isn't great. It assumes `T` is an unsigned integer, and has a size less than `u32::MAX`. For sane users,
// these will always be true. But if these assumptions don't hold, the 'min' and 'max' this reports will be wrong.
fn varuint_range_error<T>(value: u64) -> Error {
    let size = core::mem::size_of::<T>() as u32;
    let shift_count = u128::BITS - (size * 8);
    let error = InvalidDataErrorKind::OutOfRange {
        value: value as i128,
        min: 0,
        max: (u128::MAX >> shift_count) as i128,
        typename: core::any::type_name::<T>(),
    };
    error.into()
}

impl<I: InputSource> Decoder<I> {
    /// Reads between 1 and 8 bytes from the buffer and decodes a single signed integer from them.
    /// This integer must of been encoded in the variable length '[varint]' format.
    ///
    /// [varint]: https://docs.icerpc.dev/slice2/language-guide/primitive-types#variable-size-integral-types
    pub fn decode_varint<T: TryFrom<i64>>(&mut self) -> Result<T> {
        // Peek the next byte in the buffer. The lowest two bits of this byte tell us how many total bytes to read, so
        // we can pick an appropriate decoding function. This works because 'varint's are always encoded little-endian.
        // See: https://docs.icerpc.dev/slice2/encoding/primitive-types#variable-size-integral-types.
        #[rustfmt::skip] // To keep the match arms aligned for readability.
        let mut value = match self.peek_byte()? & 0b11 {
            0b00 =>  i8::decode_from(self)? as i64,
            0b01 => i16::decode_from(self)? as i64,
            0b10 => i32::decode_from(self)? as i64,
            0b11 => i64::decode_from(self)?,

            // # SAFETY
            // This match is exhaustive. There are only 4 possible values after applying a mask of `0b11` and we cover
            // all of them. This branch is mathematically impossible to hit, so we inform the compiler of this.
            _ => unsafe { core::hint::unreachable_unchecked() },
        };

        // Bit-shift the lowest 2 bits away. These stored the number of bytes to read, and are not part of the value.
        value >>= 2;
        // Try to convert the decoded value to the requested type.
        T::try_from(value).map_err(|_| varint_range_error::<T>(value))
    }

    /// Reads between 1 and 8 bytes from the buffer and decodes a single unsigned integer from them.
    /// This integer must of been encoded in the variable length '[varuint]' format.
    ///
    /// [varuint]: https://docs.icerpc.dev/slice2/language-guide/primitive-types#variable-size-integral-types
    pub fn decode_varuint<T: TryFrom<u64>>(&mut self) -> Result<T> {
        // Peek the next byte in the buffer. The lowest two bits of this byte tell us how many total bytes to read, so
        // we can pick an appropriate decoding function. This works because 'varuint's are always encoded little-endian.
        // See: https://docs.icerpc.dev/slice2/encoding/primitive-types#variable-size-integral-types.
        #[rustfmt::skip] // To keep the match arms aligned for readability.
        let mut value = match self.peek_byte()? & 0b11 {
            0b00 =>  u8::decode_from(self)? as u64,
            0b01 => u16::decode_from(self)? as u64,
            0b10 => u32::decode_from(self)? as u64,
            0b11 => u64::decode_from(self)?,

            // # SAFETY
            // This match is exhaustive. There are only 4 possible values after applying a mask of `0b11` and we cover
            // all of them. This branch is mathematically impossible to hit, so we inform the compiler of this.
            _ => unsafe { core::hint::unreachable_unchecked() },
        };

        // Bit-shift the lowest 2 bits away. These stored the number of bytes to read, and are not part of the value.
        value >>= 2;
        // Try to convert the decoded value to the requested type.
        T::try_from(value).map_err(|_| varuint_range_error::<T>(value))
    }

    /// An alias for `[decode_varuint]` to increase readability.
    pub fn decode_size(&mut self) -> Result<usize> {
        self.decode_varuint()
    }

    /// Skips any remaining tagged fields.
    pub fn skip_tagged_fields(&mut self) -> Result<()> {
        // Continue decoding tags until we hit 'TAG_END_MARKER'.
        while self.decode_varint::<i32>()? != TAG_END_MARKER {
            // Skip over the tagged field.
            let field_size = self.decode_size()?;
            self.read_byte_slice_exact(field_size)?;
        }

        Ok(())
    }
}

// =============================================================================
// Sequence type implementations
// =============================================================================

#[cfg(feature = "alloc")]
/// TODO
impl DecodeFrom for String {
    fn decode_from(decoder: &mut Decoder<impl InputSource>) -> Result<Self> {
        // Decode how many bytes are in this string, and attempt to allocate a vec with the necessary capacity.
        let length = decoder.decode_varuint()?;
        let mut vector = Vec::new();
        vector.try_reserve_exact(length)?;

        // Read 'length'-many bytes into the vector, and attempt to decode them as a utf-8 string.
        unsafe {
            debug_assert_eq!(vector.len(), 0);
            let bytes =
                core::mem::transmute::<&mut [core::mem::MaybeUninit<u8>], &mut [u8]>(vector.spare_capacity_mut());
            decoder.read_bytes_into_exact(bytes)?;
            vector.set_len(length);
        }

        let string = String::from_utf8(vector)?;
        Ok(string)
    }
}

#[cfg(feature = "alloc")]
impl<T> DecodeFrom for Vec<T>
where
    T: DecodeFrom,
{
    /// TODO
    fn decode_from(decoder: &mut Decoder<impl InputSource>) -> Result<Self> {
        // Decode how many elements are in this sequence, and attempt to allocate a vec with the necessary capacity.
        let length = decoder.decode_varuint()?;
        let mut vector = Vec::new();
        vector.try_reserve_exact(length)?;

        // Decode each element, and push them into the vector, one by one.
        for _ in 0..length {
            let element = decoder.decode()?;
            vector.push(element);
        }
        Ok(vector)
    }
}

// =============================================================================
// Dictionary type implementations
// =============================================================================

#[cfg(feature = "std")]
impl<K, V> DecodeFrom for HashMap<K, V>
where
    K: DecodeFrom + Eq + Hash,
    V: DecodeFrom,
{
    /// TODO
    fn decode_from(decoder: &mut Decoder<impl InputSource>) -> Result<Self> {
        // Decode how many entries are in this dictionary, and attempt to allocate a map with the necessary capacity.
        let length = decoder.decode_varuint()?;
        let mut map = HashMap::new();
        map.try_reserve(length)?;

        // Decode 'length'-many entries into the map.
        decode_dictionary_entries!(map, decoder, length);
        Ok(map)
    }
}

#[cfg(feature = "alloc")]
impl<K, V> DecodeFrom for BTreeMap<K, V>
where
    K: DecodeFrom + Ord,
    V: DecodeFrom,
{
    /// TODO
    fn decode_from(decoder: &mut Decoder<impl InputSource>) -> Result<Self> {
        // Decode how many entries are in this dictionary, and attempt to allocate a map with the necessary capacity.
        let length = decoder.decode_varuint()?;
        let mut map = BTreeMap::new();

        // Decode 'length'-many entries into the map.
        decode_dictionary_entries!(map, decoder, length);
        Ok(map)
    }
}

// TODO add support for optional sequences and dictionaries
