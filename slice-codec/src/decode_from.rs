// Copyright (c) ZeroC, Inc.

use crate::buffer::InputSource;
use crate::decoder::Decoder;
use crate::{Encoding, Result};

/// TODO
pub trait DecodeFrom<E: Encoding>: Sized {
    /// Decodes a value of this type from the provided decoder.
    fn decode_from(decoder: &mut Decoder<impl InputSource, E>) -> Result<Self>;
}
