// Copyright (c) ZeroC, Inc.

use crate::encoder::Encoder;
use crate::{Encoding, Result};

/// TODO
pub trait TryEncode<E: Encoding>
    where Self: Sized,
{
    /// TODO
    fn try_encode(self, encoder: &mut Encoder<E>) -> Result<()>;
}

/// TODO
pub trait TryEncodeCollection<E: Encoding, T>
    where Self: Sized,
{
    /// TODO
    fn try_encode_with_fn(self, encoder: &mut Encoder<E>, encode_fn: EncodeFn<T, E>) -> Result<()>;
}

/// TODO
pub type EncodeFn<T, E> = fn(T, &mut Encoder<E>) -> Result<()>;
