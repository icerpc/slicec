// Copyright (c) ZeroC, Inc.

use crate::decoder::Decoder;
use crate::{Encoding, Result};

/// TODO
pub trait TryDecode<E: Encoding>
    where Self: Sized,
{
    /// TODO
    fn try_decode(decoder: &mut Decoder<E>) -> Result<Self>;
}

/// TODO
pub trait TryDecodeCollection<E: Encoding, T>
    where Self: Sized,
{
    /// TODO
    fn try_decode_with_fn(decoder: &mut Decoder<E>, decode_fn: DecodeFn<T, E>) -> Result<Self>;
}

/// TODO
pub type DecodeFn<T, E> = fn(&mut Decoder<E>) -> Result<T>;
