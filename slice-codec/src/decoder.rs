// Copyright (c) ZeroC, Inc.

use crate::try_decode::TryDecode;
use crate::{Encoding, Error, Result};

const NO_LIMIT: usize = usize::MAX;

#[derive(Debug)]
pub struct Decoder<
    'a,
    #[cfg(feature = "slice2")] E: Encoding = crate::slice2::Slice2,
    #[cfg(not(feature = "slice2"))] E: Encoding,
> {
    _encoding: core::marker::PhantomData<E>,

    buffer: core::marker::PhantomData<&'a ()>, // TODO

    #[cfg(feature = "alloc")]
    total_heap_allocation_size: usize,

    #[cfg(feature = "alloc")]
    maximum_allowed_heap_allocation_size: usize,
}

impl<'a, E: Encoding> Decoder<'a, E> {
    #[cfg(feature = "alloc")]
    pub fn with_allocation_limit(mut self, limit: usize) -> Self {
        self.maximum_allowed_heap_allocation_size = limit;
        self
    }

    #[cfg(feature = "alloc")]
    pub fn with_no_allocation_limit(mut self) -> Self {
        self.maximum_allowed_heap_allocation_size = NO_LIMIT;
        self
    }

    pub fn try_decode<T: TryDecode<E>>(&mut self) -> Result<T> {
        T::try_decode(self)
    }

    pub fn peek_byte(&mut self) -> Option<&u8> {
        todo!() // TODO
    }

    pub fn read_byte(&mut self) -> Result<&'a u8> {
        todo!() // TODO
    }

    pub fn peek_bytes(&mut self, count: usize) -> &[u8] {
        todo!() // TODO
    }

    pub fn read_bytes_exact(&mut self, count: usize) -> Result<&'a [u8]> {
        todo!() // TODO
    }

    pub fn read_array_exact<const N: usize>(&mut self) -> Result<&'a [u8; N]> {
        todo!() // TODO
    }

    pub fn remaining(&mut self) -> usize {
        todo!() // TODO
    }

    #[cfg(feature = "alloc")]
    pub fn remaining_heap_allocation_size(&self) -> usize {
        if self.maximum_allowed_heap_allocation_size == NO_LIMIT {
            NO_LIMIT
        } else {
            // SAFETY: overflow is impossible because `total_heap_allocation_size <= maximum_allowed_heap_allocation_size`.
            self.maximum_allowed_heap_allocation_size - self.total_heap_allocation_size
        }
    }

    #[cfg(feature = "alloc")]
    pub fn increase_heap_allocation_total(&mut self, size: usize) -> Result<()> {
        let new_total_heap_allocation_size = self.total_heap_allocation_size.saturating_add(size);

        // If the new total allocation size is within the limit, update the total allocation size and return `Ok`,
        // otherwise return a (`HeapAllocationLimitReached`)[`Error::HeapAllocationLimitReached`] error.
        if new_total_heap_allocation_size <= self.maximum_allowed_heap_allocation_size {
            self.total_heap_allocation_size = new_total_heap_allocation_size;
            Ok(())
        } else {
            Err(Error::HeapAllocationLimitReached {
                limit: self.maximum_allowed_heap_allocation_size,
                current: self.total_heap_allocation_size,
                requested: size,
            })
        }
    }
}
