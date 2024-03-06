// Copyright (c) ZeroC, Inc.

use crate::try_decode::TryDecode;
use crate::{Encoding, Error, Result};

/// Setting this value for [`maximum_allowed_heap_allocation_size`](Decoder::maximum_allowed_heap_allocation_size)
/// disables the decoder's built-in allocation limit, allowing for it to allocate arbitrary amounts of heap memory.
///
/// This is generally a dangerous thing to do, as it allows malformed or malicious payloads to exhaust system resources.
const NO_LIMIT: usize = usize::MAX;

/// TODO
#[derive(Debug)]
pub struct Decoder<
    'a,
    #[cfg(feature = "slice2")] E: Encoding = crate::slice2::Slice2,
    #[cfg(not(feature = "slice2"))] E: Encoding,
> {
    /// Which version of the Slice encoding this decoder is using.
    _encoding: core::marker::PhantomData<E>,

    buffer_temp: core::marker::PhantomData<&'a ()>, // TODO

    /// Stores a running total of how much heap memory this decoder has allocated.
    ///
    /// This information is used to prevent malformed or malicious payloads from triggering huge allocations, or abusing
    /// system resources. If the decoder attempts to allocate memory which would cause this running total to exceed
    /// [`maximum_allowed_heap_allocation_size`](Self::maximum_allowed_heap_allocation_size), an [`Error`] is
    /// returned instead of performing the allocation.
    #[cfg(feature = "alloc")]
    total_heap_allocation_size: usize,

    /// Stores the maximum amount of heap memory this decoder is allowed to allocate.
    /// See [`total_heap_allocation_size`](Self::total_heap_allocation_size) for more information.
    #[cfg(feature = "alloc")]
    maximum_allowed_heap_allocation_size: usize,
}

impl<'a, E: Encoding> Decoder<'a, E> {
    // TODO new function here?

    /// Sets this decoder's allocation limit to the the provided value, then returns it by value.
    ///
    /// For more information on how the allocation limit works, see:
    /// [`increase_heap_allocation_total`](Self::increase_heap_allocation_total)
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer: &[u8] = &[0; 16];
    /// let decoder = Decoder::new(buffer).with_allocation_limit(16);
    ///
    /// assert_eq!(decoder.remaining_heap_allocation_size(), 16);
    /// ```
    #[cfg(feature = "alloc")]
    pub fn with_allocation_limit(mut self, limit: usize) -> Self {
        self.maximum_allowed_heap_allocation_size = limit;
        self
    }

    /// Sets this decoder's allocation limit to `factor` multiplied by the number of bytes remaining in
    /// Sets this decoder's allocation limit using the following expression: `limit = factor * remaining`.
    /// TODO
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer: &[u8] = &[0; 16];
    /// let decoder = Decoder::new(buffer).with_proportional_allocation_limit(4);
    ///
    /// assert_eq!(decoder.remaining_heap_allocation_size(), 64);
    /// ```
    ///
    /// ```
    /// let buffer: &[u8] = &[0; 12];
    /// let decoder = Decoder::new(buffer).with_proportional_allocation_limit(10);
    ///
    /// assert_eq!(decoder.remaining_heap_allocation_size(), 120);
    /// ```
    #[cfg(feature = "alloc")]
    pub fn with_proportional_allocation_limit(mut self, factor: usize) -> Self {
        // TODO self.maximum_allowed_heap_allocation_size = ...
        self
    }

    /// Disables this decoder's allocation limit check, then returns it by value.
    /// This is generally a dangerous thing to do, as it allows malformed or malicious payloads to exhaust resources.
    ///
    ///  For more information on how the allocation limit works, see:
    /// [`increase_heap_allocation_total`](Self::increase_heap_allocation_total)
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer: &[u8] = &[0; 16];
    /// let decoder = Decoder::new(buffer).with_no_allocation_limit();
    ///
    /// assert_eq!(decoder.remaining_heap_allocation_size(), usize::MAX);
    ///
    /// // Requesting to allocate a huge amount of memory succeeds...
    /// assert!(decoder.increase_heap_allocation_total(usize::MAX / 2).is_ok());
    /// // ... and we aren't any closer to hitting the limit.
    /// assert_eq!(decoder.remaining_heap_allocation_size(), usize::MAX);
    /// ```
    #[cfg(feature = "alloc")]
    pub fn with_no_allocation_limit(mut self) -> Self {
        self.maximum_allowed_heap_allocation_size = NO_LIMIT;
        self
    }

    /// Attempts to decode a value of `T` from this decoder's buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer: &[u8] = &[230, 125, 110, 101, 1];
    /// let decoder = Decoder::new(buffer);
    ///
    /// let time: i32 = decoder.try_decode().unwrap();
    /// assert_eq!(time, 1701740006);
    ///
    /// let flag: bool = decoder.try_decode().unwrap();
    /// assert_eq!(flag, true);
    ///
    /// ```
    pub fn try_decode<T: TryDecode<E>>(&mut self) -> Result<T> {
        T::try_decode(self)
    }

    /// Returns a reference to the next byte in this decoder's buffer, if present.
    /// This byte is not consumed, and the decoder's position is not advanced by calling this function.
    ///
    /// If the decoder is at end-of-buffer, `None` is returned instead.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer: &[u8] = &[7, 9];
    /// let decoder = Decoder::new(buffer);
    ///
    /// // `peek_byte` returns the same value as `read_byte`, but doesn't consume it.
    /// assert_eq!(decoder.peek_byte(), Some(&7));
    /// assert_eq!(decoder.read_byte(), Ok(&7));
    ///
    /// // Calling `peek_byte` multiple times doesn't advance the decoder's position.
    /// assert_eq!(decoder.peek_byte(), Some(&9));
    /// assert_eq!(decoder.peek_byte(), Some(&9));
    /// assert_eq!(decoder.read_byte(), Ok(&9));
    ///
    /// // `peek_byte` returns `None` once the decoder reaches end-of-buffer.
    /// assert_eq!(decoder.peek_byte(), None);
    /// assert_eq!(decoder.peek_byte(), None);
    /// assert!(decoder.read_byte().is_err());
    /// ```
    pub fn peek_byte(&mut self) -> Option<&u8> {
        todo!() // TODO
    }

    /// Returns a reference to the next byte in this decoder's buffer, if present, then advances this decoder's position
    /// by 1 (consuming the byte).
    ///
    /// If the decoder is at end-of-buffer, `Err` is returned instead.
    /// Note that certain input sources may fill with additional bytes at some point later. So even after returning an
    /// `Err`, calling `read_byte` again in the future may still yield a byte. This situation is unlikely however.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer: &[u8] = &[1, 2, 8];
    /// let decoder = Decoder::new(buffer);
    ///
    /// // `read_byte` consumes and returns bytes from the decoder's buffer, one at a time.
    /// assert_eq!(decoder.read_byte(), Ok(&1));
    /// assert_eq!(decoder.read_byte(), Ok(&2));
    /// assert_eq!(decoder.read_byte(), Ok(&8));
    ///
    /// // Until end-of-buffer is reached.
    /// assert!(decoder.read_byte().is_err());
    /// ```
    pub fn read_byte(&mut self) -> Result<&'a u8> {
        todo!() // TODO
    }

    /// Returns a slice of the next `count`-many bytes in this decoder's buffer.
    ///
    /// If there are less than `count` many bytes remaining in the buffer, this returns as many as possible.
    /// In the most extreme case (end-of-buffer), this returns an empty slice.
    ///
    /// Any returned bytes are not consumed, and the decoder's position is not advanced by calling this function.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer: &[u8] = &[1, 2, 3, 4, 5, 6];
    /// let decoder = Decoder::new(buffer);
    ///
    /// // `peek_bytes` returns the same values as `read_bytes_exact`, but doesn't consume them.
    /// assert_eq!(decoder.peek_bytes(3), &[1, 2, 3]);
    /// assert_eq!(decoder.read_bytes(3), Ok(&[1, 2, 3]));
    ///
    /// // Calling `peek_bytes` multiple times doesn't advance the decoder's position.
    /// assert_eq!(decoder.peek_bytes(2), &[4, 5]);
    /// assert_eq!(decoder.peek_bytes(2), &[4, 5]);
    ///
    /// // `peek_bytes` can return less bytes than requested if the buffer isn't large enough.
    /// assert_eq!(decoder.peek_bytes(9), &[4, 5]);
    /// assert_eq!(decoder.peek_bytes(5), &[4, 5]);
    ///
    /// assert_eq!(decoder.read_byte(), Ok(&4));
    /// assert_eq!(decoder.peek_bytes(5), &[5]);
    ///
    /// assert_eq!(decoder.read_byte(), Ok(&5));
    /// assert_eq!(decoder.peek_bytes(5), &[]);
    /// ```
    pub fn peek_bytes(&mut self, count: usize) -> &[u8] {
        todo!() // TODO
    }

    /// Returns a slice of the next `count`-many bytes in this decoder's buffer, if all present, than advances this
    /// decoder's position by `count` (consuming the bytes). The returned slice's length is guaranteed to be `count`.
    ///
    /// If there are less than `count` many bytes remaining in the buffer, this returns `Err` instead.
    /// Any remaining bytes are left unconsumed, and the decoder's position is not advanced.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer: &[u8] = &[1, 2, 3, 4, 5, 6];
    /// let decoder = Decoder::new(buffer);
    ///
    /// // `read_bytes_exact` consumes and returns exactly `count` bytes from the decoder's buffer.
    /// assert_eq!(decoder.read_bytes_exact(2), Ok(&[1, 2]));
    /// assert_eq!(decoder.read_bytes_exact(1), Ok(&[3]));
    /// assert_eq!(decoder.read_bytes_exact(0), Ok(&[]));
    /// assert_eq!(decoder.read_bytes_exact(1), Ok(&[4]));
    ///
    /// // `read_bytes_exact` returns an error if the you request more bytes than are available...
    /// assert!(decoder.read_bytes_exact(10).is_err());
    ///
    /// // ... but leaves the buffer unaffected, so any remaining bytes can still be retrieved.
    /// assert_eq!(decoder.read_bytes_exact(2), Ok(&[5, 6]));
    /// assert_eq!(decoder.remaining(), 0);
    /// ```
    pub fn read_bytes_exact(&mut self, count: usize) -> Result<&'a [u8]> {
        todo!() // TODO
    }

    /// Equivalent to [`read_bytes_exact`], but returns a reference to an array with a static length, instead of a slice
    /// with a dynamic length.
    ///
    /// This should be preferred over [`read_bytes_exact`] whenever the number of bytes to read is known at compile-time
    /// since it allows for better optimization and for bound-checks to be validated at compile-time instead of run-time.
    ///
    /// [`read_bytes_exact`]: Self::read_bytes_exact
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer: &[u8] = &[1, 2, 3, 4, 5, 6];
    /// let decoder = Decoder::new(buffer);
    ///
    /// // `read_array_exact` consumes and returns exactly `count` bytes from the decoder's buffer.
    /// assert_eq!(decoder.read_array_exact(2), Ok(&[1, 2]));
    /// assert_eq!(decoder.read_array_exact(1), Ok(&[3]));
    /// assert_eq!(decoder.read_array_exact(0), Ok(&[]));
    /// assert_eq!(decoder.read_array_exact(1), Ok(&[4]));
    ///
    /// // `read_array_exact` returns an error if the you request more bytes than are available...
    /// assert!(decoder.read_array_exact(10).is_err());
    ///
    /// // ... but leaves the buffer unaffected, so any remaining bytes can still be retrieved.
    /// assert_eq!(decoder.read_array_exact(2), Ok(&[5, 6]));
    /// assert_eq!(decoder.remaining(), 0);
    /// ```
    pub fn read_array_exact<const N: usize>(&mut self) -> Result<&'a [u8; N]> {
        todo!() // TODO
    }

    /// Returns the number of bytes remaining in this decoder's buffer.
    ///
    /// Decoders that operate over input sources that continually fill over time always return `[usize::MAX]`, since
    /// they cannot know the number of remaining bytes, and may potentially continue providing bytes indefinitely.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer: &[u8] = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
    /// let decoder = Decoder::new(buffer);
    ///
    /// assert_eq!(decoder.remaining(), 10);
    ///
    /// assert_eq!(decoder.read_bytes_exact(6), Ok(&[0, 1, 2, 3, 4, 5]));
    /// assert_eq!(decoder.remaining(), 4);
    /// ```
    pub fn remaining(&mut self) -> usize {
        todo!() // TODO
    }

    /// Returns the amount of heap memory (in bytes) this decoder can allocate before hitting its allocation limit.
    ///
    /// If this decoder's allocation limit has been disabled (see [`Self::with_no_allocation_limit`]), this always
    /// returns [`usize::MAX`].
    ///
    /// For more information on how the allocation limit works, see:
    /// [`increase_heap_allocation_total`](Self::increase_heap_allocation_total)
    ///
    /// # Examples
    ///
    /// ```
    /// // Create a decoder over an empty buffer, with an allocation limit of '16' bytes.
    /// let decoder = Decoder::new(&[]).with_allocation_limit(16);
    ///
    /// assert_eq!(decoder,remaining_heap_allocation_size(), 16);
    ///
    /// // Pretend we're about to allocate 10 bytes of heap memory.
    /// assert!(decoder.increase_heap_allocation_total(10).is_ok());
    /// assert_eq!(decoder,remaining_heap_allocation_size(), 6);
    /// ```
    #[cfg(feature = "alloc")]
    pub fn remaining_heap_allocation_size(&self) -> usize {
        if self.maximum_allowed_heap_allocation_size == NO_LIMIT {
            NO_LIMIT
        } else {
            // SAFETY: overflow is impossible because `total_heap_allocation_size <= maximum_allowed_heap_allocation_size`.
            self.maximum_allowed_heap_allocation_size - self.total_heap_allocation_size
        }
    }

    /// Any decoding logic that is about to allocate heap memory must first call this function before allocating,
    /// specifying how much memory is going to be allocated.
    ///
    /// Note that this function doesn't perform any allocation itself. It just checks whether an allocation is allowed.
    ///
    /// The decoder has an internal limit for how much heap memory it's allowed to allocate, and keeps a running total
    /// of its allocations. This function is used to update that running total.
    ///
    /// Normally, this function just updates the total and returns `Ok(())`, but if performing the allocation would put
    /// this decoder over its limit, it returns `Err` instead, signaling that the allocation should not be performed.
    ///
    /// This logic protects against malformed or malicious payloads exhausting system resources via massive allocations.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// // Mock function that attempts to allocate `capacity`-many bytes of heap memory.
    /// let allocate_vec = |capacity| {
    ///     // Ensure we can allocate `capacity` many bytes without passing the decoder's limit.
    ///     decoder.increase_heap_allocation_total(capacity)?;
    ///     // If the call didn't return `Err`, it's safe to allocate.
    ///     Ok(Vec::<u8>::with_capacity(capacity))
    /// }
    ///
    /// // Create a decoder over an empty buffer, with an allocation limit of '16' bytes.
    /// let decoder: Decoder = Decoder::new(&[]).with_allocation_limit(16);
    ///
    /// // The first time we call `allocate_vec`, `10 < 16` so the allocation succeeds.
    /// assert!(allocate_vec(10).is_ok());
    ///
    /// // The second time we call it, `20 > 16`, so the allocation fails.
    /// assert!(allocate_vec(10).is_err());
    ///
    /// // It's still safe to perform allocations after an err, if they are within the limit.
    /// assert!(allocate_vec(6).is_ok());
    /// ```
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

#[cfg(test)]
mod tests {
    // TODO
}
