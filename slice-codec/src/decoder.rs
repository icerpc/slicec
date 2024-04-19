// Copyright (c) ZeroC, Inc.

use crate::buffer::input::InputSource;
use crate::Encoding;

use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

/// TODO
pub struct Decoder<
    I: InputSource,
    #[cfg(not(feature = "slice2"))] E: Encoding,
    #[cfg(feature = "slice2")] E: Encoding = crate::slice2::Slice2,
> {
    /// Stores which [`Encoding`] this decoder is using. We store it as [`PhantomData`], because we only use this type
    /// to 'mark' the decoder, we don't actually need an instance of `E`.
    encoding: PhantomData<E>,

    /// The underlying input-source that this decoder will read bytes from.
    input: I,
}

#[cfg(feature = "slice2")]
impl<I: InputSource> Decoder<I> {
    /// TODO
    pub fn new(underlying: I) -> Self {
        Self::new_with_inferred_encoding(underlying)
    }
}

impl<I: InputSource, E: Encoding> Decoder<I, E> {
    /// TODO
    pub fn new_with_inferred_encoding(underlying: I) -> Self {
        Self {
            encoding: PhantomData,
            input: underlying,
        }
    }

    /// TODO
    #[allow(unused_variables)] // The `encoding` variable is only used for type inference.
    pub fn new_with_encoding(underlying: I, encoding: E) -> Self {
        Self::new_with_inferred_encoding(underlying)
    }

    /// TODO
    pub fn set_encoding<EPrime: Encoding>(self) -> Decoder<I, EPrime> {
        Decoder::new_with_inferred_encoding(self.input)
    }
}

// Allows users to call functions on the underlying input-source through this decoder.
impl<I: InputSource, E: Encoding> Deref for Decoder<I, E> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

// Allows users to call functions on the underlying input-source through this decoder.
impl<I: InputSource, E: Encoding> DerefMut for Decoder<I, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.input
    }
}
