// Copyright (c) ZeroC, Inc.

use crate::buffer::OutputTarget;
use crate::encode_into::EncodeInto;
use crate::Result;
use core::ops::{Deref, DerefMut};

/// TODO
pub struct Encoder<O: OutputTarget> {
    /// The underlying output-target that this encoder will write bytes to.
    output: O,
}

impl<O: OutputTarget> Encoder<O> {
    /// TODO
    pub fn new(underlying: O) -> Self {
        Self { output: underlying }
    }

    /// Attempts to encode the provided value into this encoder's underlying output-target.
    pub fn encode<T: EncodeInto>(&mut self, value: T) -> Result<()> {
        value.encode_into(self)
    }
}

// Allows users to call functions on the underlying output-target through this encoder.
impl<O: OutputTarget> Deref for Encoder<O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.output
    }
}

// Allows users to call functions on the underlying output-target through this encoder.
impl<O: OutputTarget> DerefMut for Encoder<O> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.output
    }
}
