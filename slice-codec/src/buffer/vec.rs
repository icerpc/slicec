// Copyright (c) ZeroC, Inc.

//! TODO maybe write a comment explaining this module?

use super::*;
use crate::{Error, ErrorKind, Result};
use alloc::vec::Vec;
use core::mem::MaybeUninit;
use core::{debug_assert, debug_assert_eq};

/// A wrapper around a [`Vec<u8>`] that implements [`OutputTarget`].
///
/// The implementation will automatically grow the Vec as needed.
#[derive(Debug)]
pub struct VecOutputTarget<'a> {
    /// The underlying buffer that this type wraps.
    buffer: &'a mut Vec<u8>,
}

impl<'a> VecOutputTarget<'a> {
    /// Attempts to ensure there are at least `requested` unwritten bytes available in the buffer.
    ///
    /// If there is already `requested`-many bytes in the underlying Vec's spare capacity, this is no-op.
    /// Otherwise, it will attempt to allocate (at least) `requested`-many bytes of additional capacity.
    ///
    /// If there was insufficient capacity, and the allocation failed, this will return an [`ErrorKind::UnexpectedEob`]
    /// error. Otherwise it will return `Ok`.
    ///
    /// This function is only used internally to ensure there is enough capacity before attempting a write operation.
    fn ensure_buffer_has_at_least(&mut self, requested: usize) -> Result<()> {
        // Use `try_reserve` to ensure there is sufficient space in the buffer. It will re-allocate if necessary.
        self.buffer.try_reserve(requested).map_err(|_err| {
            // If an error occurred, we wrap it in our own `UnexpectedEob` error and return it.
            let remaining = self.remaining();
            let kind = ErrorKind::UnexpectedEob { requested, remaining };

            // Remove this feature gate when the `Error` trait is moved; https://github.com/icerpc/slice-rust/issues/1.
            #[cfg(feature = "std")]
            return Error::new_with_source(kind, _err);
            #[cfg(not(feature = "std"))]
            return Error::new(kind);
        })
    }
}

impl OutputTarget for VecOutputTarget<'_> {
    fn remaining(&self) -> usize {
        self.buffer.capacity() - self.buffer.len()
    }

    fn write_byte(&mut self, byte: u8) -> Result<()> {
        self.ensure_buffer_has_at_least(1)?;

        // SAFETY: the above function call guarantees there's enough space in `self.buffer` to write a single byte.
        unsafe {
            debug_assert!(self.buffer.spare_capacity_mut().get_mut(0).is_some());
            let target = self.buffer.spare_capacity_mut().get_unchecked_mut(0);
            target.write(byte);

            let old_length = self.buffer.len();
            self.buffer.set_len(old_length + 1);
            Ok(())
        }
    }

    fn write_bytes_exact(&mut self, bytes: &[u8]) -> Result<()> {
        let count = bytes.len();
        self.ensure_buffer_has_at_least(count)?;

        // SAFETY: the above function call guarantees there's enough spare capacity in `self.buffer` to write `bytes`,
        // and we know the slice cannot overlap because the mutable borrow of `self` guarantees exclusive access.
        unsafe {
            debug_assert!(self.buffer.spare_capacity_mut().get_mut(..count).is_some());
            let target_slice = self.buffer.spare_capacity_mut().get_unchecked_mut(..count);

            debug_assert_eq!(target_slice.len(), count);
            // SAFETY: `MaybeUnit<T>` is guaranteed to have the same memory layout as `T`.
            let source: &[MaybeUninit<u8>] = core::mem::transmute(bytes);

            core::ptr::copy_nonoverlapping(source.as_ptr(), target_slice.as_mut_ptr(), count);

            let old_length = self.buffer.len();
            self.buffer.set_len(old_length + count);
            Ok(())
        }
    }

    fn write_bytes_into_reserved_exact(&mut self, reservation: &mut Reservation, bytes: &[u8]) -> Result<()> {
        // Get a mutable slice of the buffer - one that corresponds to the reserved range.
        let Some(reserved_slice) = self.buffer.get_mut(reservation.range()) else {
            let error = ErrorKind::InvalidReservation {
                buffer_len: self.buffer.len(),
                reserved_range: reservation.range(),
            };
            return Err(error.into());
        };

        // Ensure there's enough space remaining in the reservation.
        if reserved_slice.len() < bytes.len() {
            let error = ErrorKind::UnexpectedEob {
                requested: bytes.len(),
                remaining: reserved_slice.len(),
            };
            return Err(error.into());
        }

        // SAFETY: we just checked that there's enough space in `reserved_slice` to write `bytes`,
        // and we know the slices cannot overlap because the mutable borrow of `self` guarantees exclusive access.
        unsafe {
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), reserved_slice.as_mut_ptr(), bytes.len());
            reservation.0.start += bytes.len();
            Ok(())
        }
    }

    fn reserve_space(&mut self, count: usize) -> Result<Reservation> {
        self.ensure_buffer_has_at_least(count)?;

        // SAFETY: the above function call guarantees there's enough spare capacity in `self.buffer` for `count` bytes,
        // and `0x00` is a valid memory representation for a `u8`.
        unsafe {
            let pos = self.buffer.len();
            let end = pos + count;
            debug_assert!(self.buffer.spare_capacity_mut().get(..count).is_some());
            let target_offset = self.buffer.as_mut_ptr().add(pos);

            // Defensively zero the reserved memory, since `Vec` doesn't guarantee that memory between `length` and
            // `capacity` is initialized. Then advance past the reserved memory with `set_len`.
            core::ptr::write_bytes(target_offset, 0, count);
            self.buffer.set_len(end);

            Ok(Reservation(pos..end))
        }
    }
}

impl<'a> From<&'a mut Vec<u8>> for VecOutputTarget<'a> {
    /// Creates a new [`VecOutputTarget`] that wraps the provided vector.
    fn from(value: &'a mut Vec<u8>) -> Self {
        Self { buffer: value }
    }
}

// Allows users to create an [`Encoder`] directly from a vector,
// without needing to construct an intermediate [`VecOutputTarget`].
#[cfg(feature = "slice2")]
impl<'a, T> From<T> for crate::encoder::Encoder<VecOutputTarget<'a>>
    where T: Into<VecOutputTarget<'a>>,
{
    fn from(value: T) -> Self {
        crate::encoder::Encoder::new_with_inferred_encoding(value.into())
    }
}
