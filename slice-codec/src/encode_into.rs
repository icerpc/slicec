// Copyright (c) ZeroC, Inc.

use crate::buffer::output::OutputTarget;
use crate::encoder::Encoder;
use crate::{Encoding, Result};

/// TODO
pub trait EncodeInto<E: Encoding>: Sized {
    /// Encodes a value of this type with the provided encoder.
    fn encode_into(self, encoder: &mut Encoder<impl OutputTarget, E>) -> Result<()>;
}
