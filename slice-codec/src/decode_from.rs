// Copyright (c) ZeroC, Inc.

use crate::buffer::input::InputSource;
use crate::decoder::Decoder;
use crate::{Encoding, Result};

/// TODO
pub trait DecodeFrom<E: Encoding>: Sized {
    /// TODO
    fn decode_from(decoder: &mut Decoder<impl InputSource, E>) -> Result<Self>;
}
