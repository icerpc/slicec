// Copyright (c) ZeroC, Inc.

use crate::buffer::InputSource;
use crate::decoder::Decoder;
use crate::Result;

/// TODO
pub trait DecodeFrom: Sized {
    /// Decodes a value of this type from the provided decoder.
    fn decode_from(decoder: &mut Decoder<impl InputSource>) -> Result<Self>;
}

// =============================================================================
// Macros to streamline implementing `DecodeFrom` on common types.
// =============================================================================

/// This macro implements [`DecodeFrom`] on a numeric primitive type by calling `from_le_bytes` on it, which constructs
/// a new instance of the type from a series of little-endian bytes. We have to use a macro because there is no common
/// trait for numeric types, they all just happen to have this function with the same name and behavior.
///
/// This works out-of-the-box because both Rust and Slice use two's complement for representing signed integers,
/// and IEEE-754 for floating point numbers.
#[doc(hidden)]
#[macro_export]
macro_rules! implement_decode_from_on_numeric_primitive_type {
    ($ty:ty, $doc_text:literal) => {
        impl DecodeFrom for $ty {
            #[doc = $doc_text]
            fn decode_from(decoder: &mut Decoder<impl InputSource>) -> Result<Self> {
                let bytes = decoder.read_bytes_exact()?;
                Ok(Self::from_le_bytes(*bytes))
            }
        }
    };
}
pub use implement_decode_from_on_numeric_primitive_type; // Re-export the macro so it can be used in other modules.

/// This macro provides the logic for decoding a dictionary's elements, after its size has been decoded, and the
/// dictionary itself has been allocated. We have to use a macro because there is no common trait for dictionary types,
/// and this macro isn't a full implementation because of small differences in how the dictionaries must be constructed.
#[doc(hidden)]
#[macro_export]
macro_rules! decode_dictionary_entries {
    ($map:ident, $decoder:ident, $length:ident) => {
        // Decode each entry, and insert them into the map, one by one.
        for _ in 0..$length {
            let key = $decoder.decode()?;
            let value = $decoder.decode()?;
            if let Some(_duplicate) = $map.insert(key, value) {
                // TODO
                // If you insert a duplicate key into the map, it will return the old key.
                // So, if we hit this, we return  an error, because dictionary keys must be unique.
                todo!();
            }
        }
    };
}
pub use decode_dictionary_entries; // Re-export the macro so it can be used in other modules.
