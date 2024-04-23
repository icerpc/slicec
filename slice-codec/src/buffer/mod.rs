// Copyright (c) ZeroC, Inc.

//! TODO maybe write a comment explaining this module?

pub mod input;
pub mod output;

#[cfg(feature = "slice2")]
mod slice2_from_impls {
    use super::input::SliceInputSource;
    use super::output::SliceOutputTarget;
    use crate::decoder::Decoder;
    use crate::encoder::Encoder;

    #[cfg(feature = "alloc")]
    use super::output::VecOutputTarget;

    // Allows users to create an [`Encoder`] directly from a slice,
    // without needing to construct an intermediate [`SliceOutputTarget`].
    impl<'a, T> From<T> for Encoder<SliceOutputTarget<'a>>
        where T: Into<SliceOutputTarget<'a>>,
    {
        fn from(value: T) -> Self {
            Encoder::new_with_inferred_encoding(value.into())
        }
    }

    // Allows users to create an [`Encoder`] directly from a vector,
    // without needing to construct an intermediate [`VecOutputTarget`].
    #[cfg(feature = "alloc")]
    impl<'a, T> From<T> for Encoder<VecOutputTarget<'a>>
        where T: Into<VecOutputTarget<'a>>,
    {
        fn from(value: T) -> Self {
            Encoder::new_with_inferred_encoding(value.into())
        }
    }

    // Allows users to create a [`Decoder`] directly from a slice,
    // without needing to construct an intermediate [`SliceInputSource`].
    impl<'a, T> From<T> for Decoder<SliceInputSource<'a>>
        where T: Into<SliceInputSource<'a>>,
    {
        fn from(value: T) -> Self {
            Decoder::new_with_inferred_encoding(value.into())
        }
    }
}
