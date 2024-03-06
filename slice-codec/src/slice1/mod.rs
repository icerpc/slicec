// Copyright (c) ZeroC, Inc.

// These modules are private because they don't export any types, just implementations.
mod decoding;
mod encoding;

use crate::decoder::Decoder;
use crate::encoder::Encoder;
use crate::{Encoding, Error, Result};

/// TODO
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Slice1;

impl Encoding for Slice1 {
    /// TODO
    fn try_decode_size(decoder: &mut Decoder<Slice1>) -> Result<usize> {
        todo!() // TODO
    }

    /// TODO
    fn try_encode_size(size: usize, encoder: &mut Encoder<Slice1>) -> Result<()> {
        todo!() // TODO
    }
}

// Add Slice1 specific functions to the decoder.
impl Decoder<'_, Slice1> {
    // TODO
}

// Add Slice1 specific functions to the encoder.
impl Encoder<'_, Slice1> {
    // TODO
}
