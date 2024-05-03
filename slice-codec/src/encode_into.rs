// Copyright (c) ZeroC, Inc.

use crate::buffer::OutputTarget;
use crate::encoder::Encoder;
use crate::{Encoding, Result};

/// TODO
pub trait EncodeInto<E: Encoding>: Sized {
    /// Encodes a value of this type with the provided encoder.
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget, E>) -> Result<()>;
}

// =============================================================================
// Macros to streamline implementing `EncodeInto` on common types.
// =============================================================================

/// This macro implements [`EncodeInto`] on a borrowed type by copying it, then calling `encode_into` on the copied
/// value. For this implementation on `&T` to work, `T` must be [`Copy`] and implement [`EncodeInto`] itself.
#[doc(hidden)]
#[macro_export]
macro_rules! implement_encode_into_on_borrowed_type {
    ($ty:ty, $encoding:ident) => {
        impl EncodeInto<$encoding> for &$ty {
            // TODO: this generated broken links. [https://github.com/rust-lang/rust/issues/54172]
            #[doc = concat!("Delegates to [", stringify!($ty), "::encode_into].")]
            fn encode_into(self, encoder: &mut Encoder<impl OutputTarget, $encoding>) -> Result<()> {
                (*self).encode_into(encoder)
            }
        }
    };
}
pub use implement_encode_into_on_borrowed_type; // Re-export the macro so it can be used in other modules.

/// This macro implements [`EncodeInto`] on a numeric primitive type by calling `to_le_bytes` on it, which returns its
/// in-memory representation in little-endian format. We have to use a macro because there is no common trait for
/// numeric types, they all just happen to have this function with the same name and behavior.
///
/// This works out-of-the-box because both Rust and Slice use two's complement for representing signed integers,
/// and IEEE-754 for floating point numbers.
#[doc(hidden)]
#[macro_export]
macro_rules! implement_encode_into_on_numeric_primitive_type {
    ($ty:ty, $encoding:ident, $doc_text:literal) => {
        impl EncodeInto<$encoding> for $ty {
            #[doc = $doc_text]
            fn encode_into(self, encoder: &mut Encoder<impl OutputTarget, $encoding>) -> Result<()> {
                let bytes = self.to_le_bytes();
                encoder.write_bytes_exact(&bytes)
            }
        }
        implement_encode_into_on_borrowed_type!($ty, $encoding);
    };
}
pub use implement_encode_into_on_numeric_primitive_type; // Re-export the macro so it can be used in other modules.

/// This macro implements [`EncodeInto`] on a dictionary type by first encoding its length (the number of entries),
/// followed by encoding each key-value pair individually. We have to use a macro because there is no common trait for
/// dictionary types. If the dictionary entries are ordered, their order is preserved during the encoding process.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_encode_into_on_dictionary_type {
    ($ty:ident, $encoding:ident, $doc_text:literal) => {
        impl<'a, K, V> EncodeInto<$encoding> for &'a $ty<K, V>
        where
            &'a K: EncodeInto<$encoding>,
            &'a V: EncodeInto<$encoding>,
        {
            #[doc = $doc_text]
            fn encode_into(self, encoder: &mut Encoder<impl OutputTarget, $encoding>) -> Result<()> {
                let size = u64::try_from(self.len())?;
                encoder.encode_varuint(size)?;
                for (key, value) in self {
                    encoder.encode(key)?;
                    encoder.encode(value)?;
                }
                Ok(())
            }
        }
    };
}
pub use impl_encode_into_on_dictionary_type; // Re-export the macro so it can be used in other modules.
