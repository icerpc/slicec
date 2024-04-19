// Copyright (c) ZeroC, Inc.

//! TODO maybe write a comment explaining this module?

use self::input::SliceInputSource;
use self::output::{SliceOutputTarget, VecOutputTarget};
use crate::decoder::Decoder;
use crate::encoder::Encoder;

pub mod input;
pub mod output;

// Allows users to create an [`Encoder`] directly from a slice,
// without needing to construct an intermediate [`SliceOutputTarget`].
#[cfg(feature = "slice2")]
impl<'a, T> From<T> for Encoder<SliceOutputTarget<'a>>
    where T: Into<SliceOutputTarget<'a>>,
{
    fn from(value: T) -> Self {
        Encoder::new_with_inferred_encoding(value.into())
    }
}

// Allows users to create an [`Encoder`] directly from a vector,
// without needing to construct an intermediate [`VecOutputTarget`].
#[cfg(all(feature = "alloc", feature = "slice2"))]
impl<'a, T> From<T> for Encoder<VecOutputTarget<'a>>
    where T: Into<VecOutputTarget<'a>>,
{
    fn from(value: T) -> Self {
        Encoder::new_with_inferred_encoding(value.into())
    }
}

// Allows users to create a [`Decoder`] directly from a slice,
// without needing to construct an intermediate [`SliceInputSource`].
#[cfg(feature = "slice2")]
impl<'a, T> From<T> for Decoder<SliceInputSource<'a>>
    where T: Into<SliceInputSource<'a>>,
{
    fn from(value: T) -> Self {
        Decoder::new_with_inferred_encoding(value.into())
    }
}
