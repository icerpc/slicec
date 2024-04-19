// Copyright (c) ZeroC, Inc.

use crate::buffer::output::OutputTarget;
use crate::Encoding;

use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

/// TODO
pub struct Encoder<
    O: OutputTarget,
    #[cfg(not(feature = "slice2"))] E: Encoding,
    #[cfg(feature = "slice2")] E: Encoding = crate::slice2::Slice2,
> {
    /// Stores which [`Encoding`] this encoder is using. We store it as [`PhantomData`], because we only use this type
    /// to 'mark' the decoder, we don't actually need an instance of `E`.
    encoding: PhantomData<E>,

    /// The underlying output-target that this encoder will write bytes to.
    output: O,
}

#[cfg(feature = "slice2")]
impl<O: OutputTarget> Encoder<O> {
    /// TODO
    pub fn new(underlying: O) -> Self {
        Self::new_with_inferred_encoding(underlying)
    }
}

impl<O: OutputTarget, E: Encoding> Encoder<O, E> {
    /// TODO
    pub fn new_with_inferred_encoding(underlying: O) -> Self {
        Self {
            encoding: PhantomData,
            output: underlying,
        }
    }

    /// TODO
    #[allow(unused_variables)] // The `encoding` variable is only used for type inference.
    pub fn new_with_encoding(underlying: O, encoding: E) -> Self {
        Self::new_with_inferred_encoding(underlying)
    }

    /// TODO
    pub fn set_encoding<EPrime: Encoding>(self) -> Encoder<O, EPrime> {
        Encoder::new_with_inferred_encoding(self.output)
    }
}

// Allows users to call functions on the underlying output-target through this encoder.
impl<O: OutputTarget, E: Encoding> Deref for Encoder<O, E> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.output
    }
}

// Allows users to call functions on the underlying output-target through this encoder.
impl<O: OutputTarget, E: Encoding> DerefMut for Encoder<O, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.output
    }
}
