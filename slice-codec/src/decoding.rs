// Copyright (c) ZeroC, Inc.

use crate::decoder::Decoder;
use crate::try_decode::{DecodeFn, TryDecode, TryDecodeCollection};
use crate::{Encoding, Error, Result};

// We only support 'String', `Vec`, and `BTreeMap` if the `alloc` crate is available through the `alloc` feature flag.
#[cfg(feature = "alloc")]
use alloc::string::String;
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

impl<E: Encoding> TryDecode<E> for bool {
    /// Reads a single byte from the buffer and returns `false` if it's `0`, or `true` if it is `1`.
    /// if any other value is found, an error is returned instead.
    fn try_decode(decoder: &mut Decoder<E>) -> Result<Self> {
        // Try to read a byte from the buffer.
        let byte = *decoder.read_byte()?;

        // Strictly enforce the Slice spec; A bool must be encoded as either `0` or `1`.
        match byte {
            0 | 1 => Ok(byte != 0),
            _ => Err(Error::IllegalValue {
                value: byte as i128,
                desc: "bools can only have a numeric value of `0` or `1`",
            }),
        }
    }
}

impl<E: Encoding> TryDecode<E> for u8 {
    /// Reads a single byte from the buffer and returns it, as is.
    fn try_decode(decoder: &mut Decoder<E>) -> Result<Self> {
        decoder.read_byte().copied()
    }
}

/// This macro is for implementing `TryDecode` on numeric primitive types, since all their Rust counterparts implement
/// `from_le_bytes`, which returns their in-memory representations in little endian.
///
/// This is okay for decoding, because both Rust and Slice use two's complement for representing signed integers,
/// and IEEE-754 formats for floating point numbers.
macro_rules! implement_try_decode_for_primitive_numeric_type {
    ($ty:ty, $doc_text:literal, $encoding:ident$(: $($bounds:tt)+)?) => {
        impl$(<$encoding: $($bounds)+>)? TryDecode<$encoding> for $ty {
            #[doc = $doc_text]
            fn try_decode(decoder: &mut Decoder<$encoding>) -> Result<Self> {
                let bytes = decoder.read_array_exact()?;
                Ok(<$ty>::from_le_bytes(*bytes))
            }
        }
    };
}
// Re-export it so other modules in this crate can use it.
pub(crate) use implement_try_decode_for_primitive_numeric_type;

implement_try_decode_for_primitive_numeric_type!(i16, "Reads 2 bytes from the buffer and decodes a single [`i16`] from them (in two's complement little endian).", E: Encoding);
implement_try_decode_for_primitive_numeric_type!(i32, "Reads 4 bytes from the buffer and decodes a single [`i32`] from them (in two's complement little endian).", E: Encoding);
implement_try_decode_for_primitive_numeric_type!(i64, "Reads 8 bytes from the buffer and decodes a single [`i64`] from them (in two's complement little endian).", E: Encoding);
implement_try_decode_for_primitive_numeric_type!(f32, "Reads 4 bytes from the buffer and decodes a single [`f32`] from them (in IEEE 754 'binary32' little endian).", E: Encoding);
implement_try_decode_for_primitive_numeric_type!(f64, "Reads 8 bytes from the buffer and decodes a single [`f64`] from them (in IEEE 754 'binary64' little endian).", E: Encoding);

// =============================================================================
// Sequence type implementations
// =============================================================================

#[cfg(feature = "alloc")]
impl<E: Encoding> TryDecode<E> for String {
    fn try_decode(decoder: &mut Decoder<E>) -> Result<Self> {
        let length = E::try_decode_size(decoder)?;
        let buffer = decoder.read_bytes_exact(length)?.to_vec();

        String::from_utf8(buffer).map_err(|_| Error::InvalidData {
            desc: "encountered invalid utf-8 while decoding string",
        })
    }
}

#[cfg(feature = "alloc")]
impl<T, E: Encoding> TryDecodeCollection<E, T> for Vec<T> {
    fn try_decode_with_fn(decoder: &mut Decoder<E>, decode_fn: DecodeFn<T, E>) -> Result<Self> {
        let element_count = E::try_decode_size(decoder)?;
        let mut vector = Vec::with_capacity(element_count);

        for _ in 0..element_count {
            let value = decode_fn(decoder)?;
            vector.push(value);
        }
        Ok(vector)
    }
}


fn test() {
    let thing = alloc::vec![4, 5, 6];
    let decoder = todo!();
}


#[cfg(feature = "alloc")]
impl<T, E: Encoding> TryDecode<E> for Vec<T>
    where T: TryDecode<E>,
{
    fn try_decode(decoder: &mut Decoder<E>) -> Result<Self> {
        Self::try_decode_with_fn(decoder, <T>::try_decode)
    }
}

// =============================================================================
// Dictionary type implementations
// =============================================================================

#[cfg(feature = "alloc")]
macro_rules! try_decode_dictionary_body {
    () => {
        fn try_decode(decoder: &mut Decoder<E>) -> Result<Self> {
            Self::try_decode_with_fn(decoder, |decoder| {
                let key = K::try_decode(decoder)?;
                let value = V::try_decode(decoder)?;
                Ok((key, value))
            })
        }
    };
}

#[cfg(feature = "std")]
impl<K, V, E: Encoding> TryDecodeCollection<E, (K, V)> for HashMap<K, V>
    where K: Eq + Hash,
{
    fn try_decode_with_fn(decoder: &mut Decoder<E>, decode_fn: DecodeFn<(K, V), E>) -> Result<Self> {
        let entry_count = E::try_decode_size(decoder)?;
        let mut hash_map = HashMap::with_capacity(entry_count);

        for _ in 0..entry_count {
            let (key, value) = decode_fn(decoder)?;
            hash_map.insert(key, value);
        }
        Ok(hash_map)
    }
}

#[cfg(feature = "std")]
impl<K, V, E: Encoding> TryDecode<E> for HashMap<K, V>
where
    K: TryDecode<E> + Eq + Hash,
    V: TryDecode<E>,
{
    try_decode_dictionary_body!();
}

#[cfg(feature = "alloc")]
impl<K, V, E: Encoding> TryDecodeCollection<E, (K, V)> for BTreeMap<K, V>
    where K: Ord,
{
    fn try_decode_with_fn(decoder: &mut Decoder<E>, decode_fn: DecodeFn<(K, V), E>) -> Result<Self> {
        let entry_count = E::try_decode_size(decoder)?;
        let mut btree_map = BTreeMap::new();

        for _ in 0..entry_count {
            let (key, value) = decode_fn(decoder)?;
            btree_map.insert(key, value);
        }
        Ok(btree_map)
    }
}

#[cfg(feature = "alloc")]
impl<K, V, E: Encoding> TryDecode<E> for BTreeMap<K, V>
where
    K: TryDecode<E> + Ord,
    V: TryDecode<E>,
{
    try_decode_dictionary_body!();
}

// =============================================================================

#[cfg(test)]
mod tests {
    // TODO
}
