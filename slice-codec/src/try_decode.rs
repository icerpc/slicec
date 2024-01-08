// Copyright (c) ZeroC, Inc.

use crate::decoder::Decoder;
use crate::{Encoding, Result};

pub trait TryDecode<E: Encoding>
    where Self: Sized,
{
    fn try_decode(decoder: &mut Decoder<E>) -> Result<Self>;

    fn decode(decoder: &mut Decoder<E>) -> Self {
        default_error_handler(Self::try_decode(decoder))
    }
}

pub trait TryDecodeCollection<E: Encoding, T>
    where Self: Sized,
{
    fn try_decode_with_fn(decoder: &mut Decoder<E>, decode_fn: DecodeFn<T, E>) -> Result<Self>;

    fn decode_with_fn(decoder: &mut Decoder<E>, decode_fn: DecodeFn<T, E>) -> Self {
        default_error_handler(Self::try_decode_with_fn(decoder, decode_fn))
    }
}

pub type DecodeFn<T, E> = fn(&mut Decoder<E>) -> Result<T>;

#[inline(always)]
fn default_error_handler<T>(result: Result<T>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("failed to decode\n{error:?}"),
    }
}
