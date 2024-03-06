// Copyright (c) ZeroC, Inc.

use super::{Slice2, Slice2Decoder};
use crate::buf_io::bit_sequence::BitSequenceReader;
use crate::decoder::Decoder;
use crate::decoding::implement_try_decode_for_primitive_numeric_type;
use crate::try_decode::{DecodeFn, TryDecode, TryDecodeCollection};
use crate::{Encoding, Error, Result};

// We only support `Vec` and `BTreeMap` if the `alloc` crate is available through the `alloc` feature flag.
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

// We only support `HashMap` if the standard library is available through the `std` feature flag.
#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::hash::Hash;

// =============================================================================
// Fixed-length type implementations
// =============================================================================

impl TryDecode<Slice2> for i8 {
    /// Reads a single byte from the buffer and decodes it as an [`i8`] in two's complement form.
    fn try_decode(decoder: &mut Slice2Decoder) -> Result<Self> {
        // Rust signed-integers are guaranteed to use a two's complement representation in memory.
        // Casting between `u8` and `i8` is no-op, and doesn't change this representation, or the sign bit.
        let byte = decoder.read_byte()?;
        Ok(*byte as i8)
    }
}

// We use a macro to implement `TryDecode<Slice2>` on `u16`, `u32`, and `u64` via the `from_le_bytes` defined on them.
// See `implement_try_decode_for_primitive_numeric_type` for more information.
implement_try_decode_for_primitive_numeric_type!(u16, "Reads 2 bytes from the buffer and decodes a single [`u16`] from them (in little endian).", Slice2);
implement_try_decode_for_primitive_numeric_type!(u32, "Reads 4 bytes from the buffer and decodes a single [`u32`] from them (in little endian).", Slice2);
implement_try_decode_for_primitive_numeric_type!(u64, "Reads 8 bytes from the buffer and decodes a single [`u64`] from them (in little endian).", Slice2);

// =============================================================================
// Variable-length integer type implementations
// =============================================================================

impl Slice2Decoder<'_> {
    /// Reads between 1 and 8 bytes from the buffer and decodes them as a single [`i32`] in two's complement form.
    /// This is for decoding signed integers up to 32 bits long, encoded in the variable length `varint32` format.
    pub fn try_decode_varint32(&mut self) -> Result<i32> {
        // `varint32` and `varint62` use the same on-the-wire representation,
        // so we re-use the logic for decoding a `varint62` into an `i64`.
        let varint62 = self.try_decode_varint62()?;

        // Then, try to convert the `i64` into an `i32`, and return an error if it's out of range.
        varint62.try_into().map_err(|_| Error::OutOfRange {
            value: varint62 as i128,
            min: super::VARINT32_MIN as i128,
            max: super::VARINT32_MAX as i128,
            typename: "varint32",
        })
    }

    /// Reads between 1 and 8 bytes from the buffer and decodes them as a single [`u32`].
    /// This is for decoding unsigned integers up to 32 bits long, encoded in the variable length `varuint32` format.
    pub fn try_decode_varuint32(&mut self) -> Result<u32> {
        // `varuint32` and `varuint62` use the same on-the-wire representation,
        // so we re-use the logic for decoding a `varuint62` into a `u64`.
        let varuint62 = self.try_decode_varuint62()?;

        // Then, try to convert the `u64` to an `u32` and return an error if it's too large.
        varuint62.try_into().map_err(|_| Error::OutOfRange {
            value: varuint62 as i128,
            min: super::VARUINT32_MIN as i128,
            max: super::VARUINT32_MAX as i128,
            typename: "varuint32",
        })
    }

    /// Reads between 1 and 8 bytes from the buffer and decodes them as a single [`i64`] in two's complement form.
    /// This is for decoding signed integers up to 62 bits long, encoded in the variable length `varint62` format.
    pub fn try_decode_varint62(&mut self) -> Result<i64> {
        // Peek at the next byte in the buffer; the lowest two bits of this byte tell us how many total bytes to read.
        let Some(size_prefix_byte) = self.peek_byte() else {
            return Err(Error::EndOfBuffer {
                attempted: 1,
                remaining: 0,
            })
        };

        // Check the first 2 bits of `size_prefix_byte` to see how many bytes the value is encoded on, then select
        // the appropriate decoding function. Variable length integers are encoded in little endian, so this is okay.
        // For more information see https://docs.icerpc.dev/slice2/encoding/primitive-types#variable-size-integral-types
        let value = unsafe {
            // # SAFETY
            // We know this `match` is exhaustive, even if the compiler can't verify it.
            // We apply a bit-mask of `0b11` to the byte, to only check the first 2 bits.
            // There are only 4 possible values this could take, and we cover all 4 of them.
            match size_prefix_byte & 0b11 {
                0b00 =>  i8::try_decode(self)? as i64,
                0b01 => i16::try_decode(self)? as i64,
                0b10 => i32::try_decode(self)? as i64,
                0b11 => i64::try_decode(self)?,
                // Inform the compiler this is mathematically impossible to hit.
                _ => std::hint::unreachable_unchecked(),
            }
        };

        // Bit-shift the first 2 bits away. These bits only store the length of the encoded value.
        // They are not part of the value itself.
        Ok(value >> 2)
    }

    /// Reads between 1 and 8 bytes from the buffer and decodes them as a single [`u64`].
    /// This is for decoding unsigned integers up to 62 bits long, encoded in the variable length `varuint62` format.
    pub fn try_decode_varuint62(&mut self) -> Result<u64> {
        // Peek at the next byte in the buffer; the lowest two bits of this byte tell us how many total bytes to read.
        let Some(size_prefix_byte) = self.peek_byte() else {
            return Err(Error::EndOfBuffer {
                attempted: 1,
                remaining: 0,
            })
        };

        // Check the first 2 bits of `size_prefix_byte` to see how many bytes the value is encoded on, then select
        // the appropriate decoding function. Variable length integers are encoded in little endian, so this is okay.
        // For more information see https://docs.icerpc.dev/slice2/encoding/primitive-types#variable-size-integral-types
        let value = unsafe {
            // # SAFETY
            // We know this `match` is exhaustive, even if the compiler can't verify it.
            // We apply a bit-mask of `0b11` to the byte, to only check the first 2 bits.
            // There are only 4 possible values this could take, and we cover all 4 of them.
            match size_prefix_byte & 0b11 {
                0b00 =>  u8::try_decode(self)? as u64,
                0b01 => u16::try_decode(self)? as u64,
                0b10 => u32::try_decode(self)? as u64,
                0b11 => u64::try_decode(self)?,
                // Inform the compiler this is mathematically impossible to hit.
                _ => std::hint::unreachable_unchecked(),
            }
        };

        // Bit-shift the first 2 bits away. These bits only store the length of the encoded value.
        // They are not part of the value itself.
        Ok(value >> 2)
    }
}

// =============================================================================
// Optional sequence type implementations
// =============================================================================

// Auto-implement `TryDecodeCollection<Slice2, T>` for all `Vec<Option<T>>` types.
// We can't defer to `TryDecode`; element decoding isn't self-contained because of bit-sequences.
#[cfg(feature = "alloc")]
impl<T> TryDecodeCollection<Slice2, T> for Vec<Option<T>> {
    /// TODO (also comments need to go in the function itself)
    fn try_decode_with_fn(decoder: &mut Slice2Decoder, decode_fn: DecodeFn<T, Slice2>) -> Result<Self> {
        let element_count = Slice2::try_decode_size(decoder)?;
        let mut vector = Vec::with_capacity(element_count);

        let bit_sequence_size = element_count.div_ceil(8);
        let bit_sequence_buffer = decoder.read_bytes_exact(bit_sequence_size)?;
        let mut bit_sequence_reader = BitSequenceReader::new(bit_sequence_buffer, bit_sequence_size);

        for _ in 0..element_count {
            let value = match bit_sequence_reader.read_bit() {
                true => Some(decode_fn(decoder)?),
                false => None,
            };
            vector.push(value);
        }
        Ok(vector)
    }
}

// Auto-implement `TryDecode<Slice2>` for all `Vec<Option<T>>` types, where `T` already implements `TryDecode<Slice2>`.
#[cfg(feature = "alloc")]
impl<T> TryDecode<Slice2> for Vec<Option<T>>
    where T: TryDecode<Slice2>,
{
    /// TODO
    fn try_decode(decoder: &mut Slice2Decoder) -> Result<Self> {
        Self::try_decode_with_fn(decoder, <T>::try_decode)
    }
}

// =============================================================================
// Optional dictionary type implementations
// =============================================================================

/// This macro is for implementing `TryDecode<Slice2>` on 'dictionary' types, since their implementations are identical.
/// Unfortunately, Rust doesn't have a `Dictionary` trait that could be used for a blanket implementation instead.
#[cfg(feature = "std")]
macro_rules! try_decode_optional_dictionary_body {
    () => {
        fn try_decode(decoder: &mut Slice2Decoder) -> Result<Self> {
            // Call the `try_decode_with_fn` that's already implemented, which specifies how to decode each element.
            Self::try_decode_with_fn(decoder, |decoder| {
                // We decode a bool to determine if the next value is set; `true` means set, `false` means not-set.
                // This is equivalent to reading a single-element bit-sequence.
                let is_set = bool::try_decode(decoder)?;

                // Decode the key type, then decode the value type if we expect it to be set.
                let key = K::try_decode(decoder)?;
                let value = match is_set {
                    true => Some(V::try_decode(decoder)?),
                    false => None,
                };
                Ok((key, value))
            })
        }
    };
}

// Auto-implement `TryDecode<Slice2>` for all `HashMaps` with an `Option` value type,
// where the key type, and the inner value type both already implement `TryDecode<Slice2>`.
#[cfg(feature = "std")]
impl<K, V> TryDecode<Slice2> for HashMap<K, Option<V>>
where
    K: TryDecode<Slice2> + Eq + Hash,
    V: TryDecode<Slice2>,
{
    // TODO (or maybe put this in the macro?)
    try_decode_optional_dictionary_body!();
}

// Auto-implement `TryDecode<Slice2>` for all `BTreeMaps` with an `Option` value type,
// where the key type, and the inner value type both already implement `TryDecode<Slice2>`.
#[cfg(feature = "alloc")]
impl<K, V> TryDecode<Slice2> for BTreeMap<K, Option<V>>
where
    K: TryDecode<Slice2> + Ord,
    V: TryDecode<Slice2>,
{
    // TODO (or maybe put this in the macro?)
    try_decode_optional_dictionary_body!();
}

// =============================================================================

#[cfg(test)]
mod tests {
    // TODO
}
