// Copyright (c) ZeroC, Inc.

use crate::encoder::Encoder;
use crate::{Encoding, Result};

pub trait TryEncode<E: Encoding>
    where Self: Sized,
{
    fn try_encode(self, encoder: &mut Encoder<E>) -> Result<()>;

    fn encode(self, encoder: &mut Encoder<E>) {
        default_error_handler(self.try_encode(encoder))
    }
}

pub trait TryEncodeCollection<E: Encoding, T>
    where Self: Sized,
{
    fn try_encode_with_fn(self, encoder: &mut Encoder<E>, encode_fn: EncodeFn<T, E>) -> Result<()>;

    fn encode_with_fn(self, encoder: &mut Encoder<E>, encode_fn: EncodeFn<T, E>) {
        default_error_handler(self.try_encode_with_fn(encoder, encode_fn))
    }
}

pub type EncodeFn<T, E> = fn(T, &mut Encoder<E>) -> Result<()>;

#[inline(always)]
fn default_error_handler<T>(result: Result<T>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("failed to encode\n{error:?}"),
    }
}
