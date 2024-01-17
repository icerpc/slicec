// Copyright (c) ZeroC, Inc.

use crate::encoder::Encoder;
use crate::{Encoding, Result};

pub trait TryEncode<E: Encoding>
    where Self: Sized,
{
    fn try_encode(self, encoder: &mut Encoder<E>) -> Result<()>;
}

pub trait TryEncodeCollection<E: Encoding, T>
    where Self: Sized,
{
    fn try_encode_with_fn(self, encoder: &mut Encoder<E>, encode_fn: EncodeFn<T, E>) -> Result<()>;
}

pub type EncodeFn<T, E> = fn(T, &mut Encoder<E>) -> Result<()>;
