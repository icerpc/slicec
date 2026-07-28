// Copyright (c) ZeroC, Inc.

//! TODO maybe write a comment explaining this module?

use super::*;
use crate::{ErrorKind, Result};
use core::{debug_assert, debug_assert_eq};

/// A wrapper around a `&mut [u8]` that implements [`OutputTarget`].
#[derive(Debug)]
pub struct SliceOutputTarget<'a> {
    /// The underlying buffer that this type wraps.
    buffer: &'a mut [u8],
    /// Tracks the current position in the buffer that is being written to.
    pos: usize,
}

impl<'a> SliceOutputTarget<'a> {
    /// Checks whether there are at least `requested` unwritten bytes left in the buffer.
    /// If there are, this returns `Ok`, and if there aren't this returns an [`ErrorKind::UnexpectedEob`] error.
    ///
    /// This function is only used internally to ensure a particular write operation is safe to attempt.
    fn does_buffer_have_at_least(&self, requested: usize) -> Result<()> {
        let remaining = self.remaining();
        if remaining < requested {
            let error = ErrorKind::UnexpectedEob { requested, remaining };
            Err(error.into())
        } else {
            Ok(())
        }
    }
}

impl OutputTarget for SliceOutputTarget<'_> {
    fn remaining(&self) -> usize {
        self.buffer.len() - self.pos
    }

    fn write_byte(&mut self, byte: u8) -> Result<()> {
        self.does_buffer_have_at_least(1)?;

        // SAFETY: the above function call guarantees there's enough space in `self.buffer` to write a single byte.
        unsafe {
            debug_assert!(self.buffer.get_mut(self.pos).is_some());
            *self.buffer.get_unchecked_mut(self.pos) = byte;
            self.pos += 1;
            Ok(())
        }
    }

    fn write_bytes_exact(&mut self, bytes: &[u8]) -> Result<()> {
        let count = bytes.len();
        self.does_buffer_have_at_least(count)?;

        // SAFETY: the above function call guarantees there's enough space in `self.buffer` to write `bytes`,
        // and we know the slices cannot overlap because the mutable borrow of `self` guarantees exclusive access.
        unsafe {
            let end = self.pos + count;
            debug_assert!(self.buffer.get_mut(self.pos..end).is_some());
            let target_slice = self.buffer.get_unchecked_mut(self.pos..end);
            debug_assert_eq!(target_slice.len(), count);

            core::ptr::copy_nonoverlapping(bytes.as_ptr(), target_slice.as_mut_ptr(), count);
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

impl<'a> From<&'a mut [u8]> for SliceOutputTarget<'a> {
    /// Creates a new [`SliceOutputTarget`] that wraps the provided buffer.
    fn from(value: &'a mut [u8]) -> Self {
        Self { buffer: value, pos: 0 }
    }
}

impl<'a, const N: usize> From<&'a mut [u8; N]> for SliceOutputTarget<'a> {
    /// Creates a new [`SliceOutputTarget`] that wraps the provided array.
    fn from(value: &'a mut [u8; N]) -> Self {
        Self {
            buffer: value.as_mut_slice(),
            pos: 0,
        }
    }
}

// Allows users to create an [`Encoder`] directly from a slice,
// without needing to construct an intermediate [`SliceOutputTarget`].
impl<'a, T> From<T> for crate::encoding::encoder::Encoder<SliceOutputTarget<'a>>
where
    T: Into<SliceOutputTarget<'a>>,
{
    fn from(value: T) -> Self {
        crate::encoding::encoder::Encoder::new(value.into())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Verifies that [`does_buffer_have_at_least`] returns the correct number of remaining bytes in the buffer
    /// when the remaining bytes number are greater than or equal to the number of requested bytes.
    #[test]
    fn does_buffer_has_at_least_returns_ok() {
        // Arrange
        let mut buffer = [115, 108, 105, 99, 101];
        let target = SliceOutputTarget::from(buffer.as_mut_slice());

        // Act
        let result = target.does_buffer_have_at_least(5);

        // Assert
        assert!(result.is_ok());
    }

    /// Verifies that [`does_buffer_have_at_least`] returns an error when the remaining bytes number are less than
    /// the number of requested bytes.
    #[test]
    fn does_buffer_have_at_least_returns_error() {
        // Arrange
        let mut buffer = [115, 108, 105, 99, 101];
        let target = SliceOutputTarget::from(buffer.as_mut_slice());

        // Act
        let result = target.does_buffer_have_at_least(6);

        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err().kind(), ErrorKind::UnexpectedEob {
            requested: 6,
            remaining: 5
        }));
    }

    /// Verifies that [`write_byte`] writes the correct byte to the buffer and advances the position.
    #[test]
    fn write_byte_writes_correct_byte() {
        // Arrange
        let mut buffer = [0; 5];
        let mut target = SliceOutputTarget::from(buffer.as_mut_slice());

        // Act
        let result = target.write_byte(115);

        // Assert
        assert!(result.is_ok());
        assert_eq!(target.buffer, [115, 0, 0, 0, 0]);
        assert_eq!(target.pos, 1);
        assert_eq!(target.remaining(), 4);
    }

    /// Verifies that [`write_bytes_exact`] writes the correct bytes to the buffer and advances the position.
    #[test]
    fn write_bytes_exact_writes_correct_bytes() {
        // Arrange
        let mut buffer = [0; 5];
        let mut target = SliceOutputTarget::from(buffer.as_mut_slice());

        // Act
        let result = target.write_bytes_exact(&[115, 108, 105, 99, 101]);

        // Assert
        assert!(result.is_ok());
        assert_eq!(target.buffer, [115, 108, 105, 99, 101]);
        assert_eq!(target.pos, 5);
        assert_eq!(target.remaining(), 0);
    }

    /// Verifies that [`reserve_space`] reserves the correct number of bytes in the buffer and advances the
    /// position past the reserved space so that the next write operation will not write into the reserved space.
    #[test]
    fn reserve_space_reserves_correct_space() {
        // Arrange
        let mut buffer = [0; 5];
        let mut target = SliceOutputTarget::from(buffer.as_mut_slice());

        // Act
        let reserve_result = target.reserve_space(3);
        let write_result = target.write_byte(99);

        // Assert
        assert!(reserve_result.is_ok());
        assert!(write_result.is_ok());

        assert_eq!(reserve_result.unwrap().0, 0..3);
        assert_eq!(target.pos, 4);
        assert_eq!(target.remaining(), 1);
        assert_eq!(target.buffer, [0, 0, 0, 99, 0]);
    }

    /// Verifies that [`write_bytes_into_reserved_exact`] writes the correct bytes to the reserved space in the
    /// buffer and does not advance the position past the reserved space.
    #[test]
    fn write_bytes_into_reserved_exact_writes_correct_bytes() {
        // Arrange
        let mut buffer = [0; 5];
        let mut target = SliceOutputTarget::from(buffer.as_mut_slice());

        // Should advance the position to 3.
        let mut reservation = target.reserve_space(3).unwrap();

        // Write a byte to ensure the position is advanced.
        let _ = target.write_bytes_exact(&[99]);

        // Act
        let result = target.write_bytes_into_reserved_exact(&mut reservation, &[115, 108, 105]);

        // Write a byte to ensure the position was not advanced.
        let _ = target.write_byte(101);

        // Assert
        assert!(result.is_ok());
        assert_eq!(target.buffer, [115, 108, 105, 99, 101]);
        assert_eq!(target.pos, 5);
        assert_eq!(target.remaining(), 0);
    }
}
