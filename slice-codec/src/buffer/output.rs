// Copyright (c) ZeroC, Inc.

//! TODO maybe write a comment explaining this module?

use crate::{ErrorKind, Result};

use core::borrow::BorrowMut;
use core::ops::Range;
use core::{debug_assert, debug_assert_eq};

#[cfg(feature = "alloc")]
use crate::Error;
#[cfg(feature = "alloc")]
use core::mem::MaybeUninit;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// A trait for types that can be written to by a [Slice encoder](crate::encoder::Encoder).
pub trait OutputTarget {
    /// Attempts to write the provided bytes into this target.
    ///
    /// This function will not return until either all the provided bytes have been written, or an unrecoverable error
    /// has occurred. If such an error occurs, no guarantees are made about how many bytes were written, or the state
    /// of the underlying target.
    fn write_bytes_exact(&mut self, bytes: &[u8]) -> Result<()>;

    /// Attempts to write the provided bytes into a reserved chunk of memory within this target.
    ///
    /// This is used alongside [`Self::reserve_space`] to write data into the target without appending it at the end.
    /// However, this function cannot be used to write to arbitrary positions in the target; reservations must be made
    /// in advance.
    ///
    /// It is legal to call this function multiple times with the same [`Reservation`], but as bytes are written,
    /// the reservation will be shrunk accordingly (starting from the front) to ensure bytes are never over-written.
    ///
    /// Like [`Self::write_bytes_exact`], this function will not return until either all the provided bytes have been
    /// written, or an unrecoverable error has occurred. If such an error occurs, no guarantees are made about how many
    /// bytes were written, or the state of the underlying target.
    fn write_bytes_into_reserved_exact(&mut self, reservation: &mut Reservation, bytes: &[u8]) -> Result<()>;

    /// Reserves a chunk of memory in the target that can be written to later, and advances past it.
    ///
    /// The target's cursor is advanced by `count`-many bytes (so it points to the end of the reservation) meaning
    /// additional writes to this target will continue normally outside the reserved memory.
    ///
    /// It returns a typed range ([`Reservation`]) specifying the beginning and end of this reserved memory. This memory
    /// can written to later by passing the returned [`Reservation`] into [`Self::write_bytes_into_reserved_exact`].
    ///
    /// If the underlying target has insufficient memory (and couldn't allocate more) an error is returned instead.
    fn reserve_space(&mut self, count: usize) -> Result<Reservation>;
}

/// Represents a span of bytes that have been reserved in an [`OutputTarget`].
/// See [`OutputTarget::reserve_space`].
#[derive(Debug)]
#[must_use]
pub struct Reservation(Range<usize>);

impl Reservation {
    /// Returns a [`Range`] corresponding to the byte positions held by this [`Reservation`].
    fn range(&self) -> Range<usize> {
        self.0.clone()
    }
}

/// A wrapper around a `&mut [u8]` that implements [`OutputTarget`].
#[derive(Debug)]
pub struct SliceOutputTarget<'a> {
    /// The underlying buffer that this type wraps.
    buffer: &'a mut [u8],
    /// Tracks the current position in the buffer that is being written to.
    pos: usize,
}

impl<'a> SliceOutputTarget<'a> {
    fn does_buffer_have_at_least(&self, requested: usize) -> Result<()> {
        let remaining = self.buffer.len() - self.pos;
        if remaining < requested {
            let error = ErrorKind::UnexpectedEob { requested, remaining };
            Err(error.into())
        } else {
            Ok(())
        }
    }
}

impl OutputTarget for SliceOutputTarget<'_> {
    fn write_bytes_exact(&mut self, bytes: &[u8]) -> Result<()> {
        self.does_buffer_have_at_least(bytes.len())?;

        // SAFETY: the above function call guarantees there's enough space in `self.buffer` to write `bytes`,
        // and we know the slices cannot overlap because the mutable borrow of `self` guarantees exclusive access.
        unsafe {
            let end = self.pos + bytes.len();
            debug_assert!(self.buffer.get_mut(self.pos..end).is_some());
            let target_slice = self.buffer.get_unchecked_mut(self.pos..end);
            debug_assert_eq!(target_slice.len(), bytes.len());

            core::ptr::copy_nonoverlapping(bytes.as_ptr(), target_slice.as_mut_ptr(), bytes.len());
            self.pos = end;
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
        self.does_buffer_have_at_least(count)?;

        self.pos += count;
        Ok(Reservation((self.pos - count)..self.pos))
    }
}

impl<'a, T> From<&'a mut T> for SliceOutputTarget<'a>
    where T: BorrowMut<[u8]> + ?Sized
{
    /// Creates a new [`SliceOutputTarget`] that wraps the provided buffer.
    fn from(value: &'a mut T) -> Self {
        Self {
            buffer: value.borrow_mut(),
            pos: 0,
        }
    }
}

/// A wrapper around a [`Vec<u8>`] that implements [`OutputTarget`].
///
/// The implementation will automatically grow the Vec as needed.
#[cfg(feature = "alloc")]
#[derive(Debug)]
pub struct VecOutputTarget<'a> {
    /// The underlying buffer that this type wraps.
    buffer: &'a mut Vec<u8>,
}

#[cfg(feature = "alloc")]
impl<'a> VecOutputTarget<'a> {
    fn ensure_buffer_has_at_least(&mut self, requested: usize) -> Result<()> {
        // Use `try_reserve` to ensure there is sufficient space in the buffer. It will re-allocate if necessary.
        self.buffer.try_reserve(requested).map_err(|_err| {
            // If an error occurred, we wrap it in our own `UnexpectedEob` error and return it.
            let remaining = self.buffer.capacity() - self.buffer.len();
            let kind = ErrorKind::UnexpectedEob { requested, remaining };

            // Remove this feature gate when the `Error` trait is moved; https://github.com/icerpc/slice-rust/issues/1.
            #[cfg(feature = "std")]
            return Error::new_with_source(kind, _err);
            #[cfg(not(feature = "std"))]
            return Error::new(kind);
        })
    }
}

#[cfg(feature = "alloc")]
impl OutputTarget for VecOutputTarget<'_> {
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

#[cfg(feature = "alloc")]
impl<'a> From<&'a mut Vec<u8>> for VecOutputTarget<'a> {
    /// Creates a new [`VecOutputTarget`] that wraps the provided vector.
    fn from(value: &'a mut Vec<u8>) -> Self {
        Self { buffer: value }
    }
}

// TODO we need to write more comprehensive tests
#[cfg(test)]
mod tests {
    use super::*;

    mod slice_output_target {}

    #[cfg(feature = "alloc")]
    mod vec_output_target {
        use super::*;

        // TODO test the case where `ensure_target_has_at_least` fails when the `Allocator` API is stabilized.
        // See https://github.com/icerpc-slice-rust/issues/2

        #[test]
        fn write_bytes_exact() {
            let mut buffer = Vec::with_capacity(8);
            let mut output = VecOutputTarget::from(&mut buffer);
            assert_eq!(output.buffer.len(), 0);
            assert_eq!(output.buffer.capacity(), 8);

            output.write_bytes_exact(&[8, 5, 3, 100]).unwrap();
            assert_eq!(output.buffer.len(), 4);
            assert_eq!(output.buffer.capacity(), 8);
            assert_eq!(output.buffer.as_slice(), &[8, 5, 3, 100]);

            output.write_bytes_exact(&[1, 2, 3, 4, 5, 6]).unwrap();
            let new_capacity = output.buffer.capacity();
            assert_eq!(output.buffer.len(), 10);
            assert!(new_capacity >= 10); // We don't know how much it'll grow.
            assert_eq!(output.buffer.as_slice(), &[8, 5, 3, 100, 1, 2, 3, 4, 5, 6]);

            output.write_bytes_exact(&[]).unwrap();
            assert_eq!(output.buffer.len(), 10);
            assert_eq!(output.buffer.capacity(), new_capacity);
            assert_eq!(output.buffer.as_slice(), &[8, 5, 3, 100, 1, 2, 3, 4, 5, 6]);
        }

        #[test]
        fn write_bytes_exact_into_reserved() {
            let mut buffer = Vec::with_capacity(8);
            let mut output = VecOutputTarget::from(&mut buffer);
            assert_eq!(output.buffer.len(), 0);
            assert_eq!(output.buffer.capacity(), 8);

            // Reserve some space at the start of the buffer we'll write to later.
            let mut reservation = output.reserve_space(6).unwrap();
            assert_eq!(output.buffer.len(), 6);
            assert_eq!(output.buffer.capacity(), 8);
            assert_eq!(output.buffer.as_slice(), &[0, 0, 0, 0, 0, 0]);
            assert_eq!(reservation.0.start, 0);
            assert_eq!(reservation.0.end, 6);

            // Continue writing past the reserved section.
            output.write_bytes_exact(&[79, 79]).unwrap();
            assert_eq!(output.buffer.len(), 8);
            assert_eq!(output.buffer.capacity(), 8);
            assert_eq!(output.buffer.as_slice(), &[0, 0, 0, 0, 0, 0, 79, 79]);

            // Update the first 4 bytes of the reservation.
            output.write_bytes_into_reserved_exact(&mut reservation, &[1, 2, 3, 4]).unwrap();
            assert_eq!(output.buffer.len(), 8);
            assert_eq!(output.buffer.capacity(), 8);
            assert_eq!(output.buffer.as_slice(), &[1, 2, 3, 4, 0, 0, 79, 79]);
            assert_eq!(reservation.0.start, 4);
            assert_eq!(reservation.0.end, 6);

            // Write more stuff to the end.
            output.write_bytes_exact(&[0, 1, 0, 1]).unwrap();
            let new_capacity = output.buffer.capacity(); // We don't know how much it'll grow.
            assert_eq!(output.buffer.len(), 12);
            assert!(output.buffer.capacity() >= 12);
            assert_eq!(output.buffer.as_slice(), &[1, 2, 3, 4, 0, 0, 79, 79, 0, 1, 0, 1]);

            // Finish writing the reservation.
            output.write_bytes_into_reserved_exact(&mut reservation, &[0, 6]).unwrap();
            assert_eq!(output.buffer.len(), 12);
            assert_eq!(output.buffer.capacity(), new_capacity);
            assert_eq!(output.buffer.as_slice(), &[1, 2, 3, 4, 0, 6, 79, 79, 0, 1, 0, 1]);
            assert_eq!(reservation.0.start, 6);
            assert_eq!(reservation.0.end, 6);
        }
    }
}
