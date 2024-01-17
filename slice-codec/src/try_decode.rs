// Copyright (c) ZeroC, Inc.

use crate::decoder::Decoder;
use crate::{Encoding, Result};

pub trait TryDecode<E: Encoding>
    where Self: Sized,
{
    fn try_decode(decoder: &mut Decoder<E>) -> Result<Self>;
}

pub trait TryDecodeCollection<E: Encoding, T>
    where Self: Sized,
{
    fn try_decode_with_fn(decoder: &mut Decoder<E>, decode_fn: DecodeFn<T, E>) -> Result<Self>;
}

pub type DecodeFn<T, E> = fn(&mut Decoder<E>) -> Result<T>;
