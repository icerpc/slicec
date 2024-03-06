// Copyright (c) ZeroC, Inc.

use crate::encoder::Encoder;
use crate::try_encode::{EncodeFn, TryEncode, TryEncodeCollection};
use crate::{Encoding, Result};

// We only support `BTreeMap` if the `alloc` crate is available through the `alloc` feature flag.
#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

// We only support `HashMap` if the standard library is available through the `std` feature flag.
#[cfg(feature = "std")]
use std::collections::HashMap;

// =============================================================================
// Fixed-length type implementations
// =============================================================================

/// This macro is for implementing `TryEncode` on borrows of primitive types.
/// The actual implementations are done on the types directly, since it's conventional to pass these types 'by value'.
/// But we also want it to be possible to use borrowed values for API consistency. Hence this macro.
///
/// The macro works by implementing `TryEncode` on `&T`, and the implementation just delegates to `T::TryEncode`.
macro_rules! implement_try_encode_on_borrowed_type {
    ($ty:ty, $encoding:ident$(: $($bounds:tt)+)?) => {
        impl$(<$encoding: $($bounds)*>)? TryEncode<$encoding> for &$ty {
            #[doc = concat!("Delegates to [", stringify!($ty), "::try_encode].")]                                       // TODO OPEN AN ISSUE, THIS GENERATED BOKRED LINKS [https://github.com/rust-lang/rust/issues/54172]
            #[inline(always)]
            fn try_encode(self, encoder: &mut Encoder<$encoding>) -> Result<()> {
                (*self).try_encode(encoder)
            }
        }
    };
}
// Re-export it so other modules in this crate can use it.
pub(crate) use implement_try_encode_on_borrowed_type;

impl<E: Encoding> TryEncode<E> for bool {
    /// Encodes a value of `0` for `false` or a value of `1` for true, on a single byte.
    fn try_encode(self, encoder: &mut Encoder<E>) -> Result<()> {
        // In memory, bools are guaranteed to be `0` for `false` and `1` for `true`.
        encoder.write_byte(self as u8)
    }
}
implement_try_encode_on_borrowed_type!(bool, E: Encoding);

impl<E: Encoding> TryEncode<E> for u8 {
    /// Writes this byte directly to the buffer, as is.
    fn try_encode(self, encoder: &mut Encoder<E>) -> Result<()> {
        encoder.write_byte(self)
    }
}
implement_try_encode_on_borrowed_type!(u8, E: Encoding);

/// This macro is for implementing `TryEncode` on numeric primitive types, since all their Rust counterparts implement
/// `to_le_bytes`, which returns their in-memory representations in little endian.
///
/// This is okay for encoding because both Rust and Slice use two's complement for representing signed integers,
/// and IEEE-754 formats for floating point numbers.
macro_rules! implement_try_encode_for_primitive_numeric_type {
    ($ty:ty, $doc_text:literal, $encoding:ident$(: $($bounds:tt)+)?) => {
        impl$(<$encoding: $($bounds)+>)? TryEncode<$encoding> for $ty {
            #[doc = $doc_text]
            fn try_encode(self, encoder: &mut Encoder<$encoding>) -> Result<()> {
                let bytes = self.to_le_bytes();
                encoder.write_bytes_exact(&bytes)
            }
        }
        // Implement `TryEncode` for `&ty`.
        implement_try_encode_on_borrowed_type!($ty, $encoding$(: $($bounds)*)?);
    };
}
// Re-export it so other modules in this crate can use it.
pub(crate) use implement_try_encode_for_primitive_numeric_type;

implement_try_encode_for_primitive_numeric_type!(i16, "Encodes this [`i16`] on 2 bytes (little endian) in two's complement form", E: Encoding);
implement_try_encode_for_primitive_numeric_type!(i32, "Encodes this [`i32`] on 4 bytes (little endian) in two's complement form", E: Encoding);
implement_try_encode_for_primitive_numeric_type!(i64, "Encodes this [`i64`] on 8 bytes (little endian) in two's complement form", E: Encoding);
implement_try_encode_for_primitive_numeric_type!(f32, "Encodes this [`f32`] on 4 bytes (little endian) using the \"binary32\" representation defined in IEEE 754-2008", E: Encoding);
implement_try_encode_for_primitive_numeric_type!(f64, "Encodes this [`f64`] on 8 bytes (little endian) using the \"binary64\" representation defined in IEEE 754-2008", E: Encoding);

// =============================================================================
// Sequence type implementations
// =============================================================================

impl<E: Encoding> TryEncode<E> for &str {
    fn try_encode(self, encoder: &mut Encoder<E>) -> Result<()> {
        E::try_encode_size(self.len(), encoder)?;
        encoder.write_bytes_exact(self.as_bytes())
    }
}

impl<'a, T, E: Encoding> TryEncodeCollection<E, &'a T> for &'a [T] {
    fn try_encode_with_fn(self, encoder: &mut Encoder<E>, encode_fn: EncodeFn<&'a T, E>) -> Result<()> {
        E::try_encode_size(self.len(), encoder)?;
        for element in self {
            encode_fn(element, encoder)?;
        }
        Ok(())
    }
}

impl<'a, T, E: Encoding> TryEncode<E> for &'a [T]
    where &'a T: TryEncode<E>,
{
    fn try_encode(self, encoder: &mut Encoder<E>) -> Result<()> {
        self.try_encode_with_fn(encoder, <&T>::try_encode)
    }
}

// =============================================================================
// Dictionary type implementations
// =============================================================================

#[cfg(feature = "alloc")]
macro_rules! impl_try_encode_collection_for_dictionary_type {
    ($type_name:ident) => {
        impl<'a, K, V, E: Encoding> TryEncodeCollection<E, (&'a K, &'a V)> for &'a $type_name<K, V> {
            fn try_encode_with_fn(self, encoder: &mut Encoder<E>, encode_fn: EncodeFn<(&'a K, &'a V), E>) -> Result<()> {
                E::try_encode_size(self.len(), encoder)?;
                for entry in self {
                    encode_fn(entry, encoder)?;
                }
                Ok(())
            }
        }
    };
}

#[cfg(feature = "alloc")]
macro_rules! impl_try_encode_for_dictionary_type {
    ($type_name:ident) => {
        impl<'a, K, V, E: Encoding> TryEncode<E> for &'a $type_name<K, V>
        where
            &'a K: TryEncode<E>,
            &'a V: TryEncode<E>,
        {
            fn try_encode(self, encoder: &mut Encoder<E>) -> Result<()> {
                self.try_encode_with_fn(encoder, |(key, value), encoder| {
                    key.try_encode(encoder)?;
                    value.try_encode(encoder)?;
                    Ok(())
                })
            }
        }
    };
}

#[cfg(feature = "std")]
impl_try_encode_collection_for_dictionary_type!(HashMap);
#[cfg(feature = "std")]
impl_try_encode_for_dictionary_type!(HashMap);

#[cfg(feature = "alloc")]
impl_try_encode_collection_for_dictionary_type!(BTreeMap);
#[cfg(feature = "alloc")]
impl_try_encode_for_dictionary_type!(BTreeMap);

#[cfg(test)]
mod tests {
    // TODO
}
