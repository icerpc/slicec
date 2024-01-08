// Copyright (c) ZeroC, Inc.

use crate::try_encode::TryEncode;
use crate::{Encoding, Error, Result};

#[derive(Debug)]
pub struct Encoder<'a, E: Encoding> {
    _encoding: core::marker::PhantomData<E>,

    buffer: core::marker::PhantomData<&'a ()>, // TODO
}

impl<'a, E: Encoding> Encoder<'a, E> {
    pub fn try_encode<T: TryEncode<E>>(&mut self, value: T) -> Result<()> {
        value.try_encode(self)
    }

    pub fn write_byte(&mut self, data: u8) -> Result<()> {
        todo!() // TODO
    }

    pub fn write_bytes_exact(&mut self, data: &[u8]) -> Result<()> {
        todo!() // TODO
    }

    pub fn reserve(&mut self, count: usize) -> Result<&'a mut [u8]> {
        todo!() // TODO
    }
}
