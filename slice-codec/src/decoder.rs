// Copyright (c) ZeroC, Inc.

use crate::buffer::InputSource;
use crate::decode_from::DecodeFrom;
use crate::Result;
use core::ops::{Deref, DerefMut};

/// TODO
pub struct Decoder<I: InputSource> {
    /// The underlying input-source that this decoder will read bytes from.
    input: I,
}

impl<I: InputSource> Decoder<I> {
    /// TODO
    pub fn new(underlying: I) -> Self {
        Self { input: underlying }
    }

    /// Attempts to decode a value of the specified type from this decoder's underlying input-source.
    pub fn decode<T: DecodeFrom>(&mut self) -> Result<T> {
        T::decode_from(self)
    }
}

// Allows users to call functions on the underlying input-source through this decoder.
impl<I: InputSource> Deref for Decoder<I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

// Allows users to call functions on the underlying input-source through this decoder.
impl<I: InputSource> DerefMut for Decoder<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.input
    }
}
