// Copyright (c) ZeroC, Inc.

//! TODO maybe write a comment explaining this module?

use super::*;
use crate::{ErrorKind, Result};
use core::borrow::Borrow;
use core::{debug_assert, debug_assert_eq};

/// A wrapper around a `&[u8]` that implements [`InputSource`].
#[derive(Debug)]
pub struct SliceInputSource<'a> {
    /// The underlying buffer that this type wraps.
    buffer: &'a [u8],
    /// Tracks the current position in the buffer that is being read from.
    pos: usize,
}

impl<'a> SliceInputSource<'a> {
    /// Checks whether there are at least `requested` unread bytes left in the buffer.
    /// If there are, this returns `Ok`, and if there aren't this returns an [`ErrorKind::UnexpectedEob`] error.
    ///
    /// This function is only used internally to ensure a particular read operation is safe to attempt.
    fn does_buffer_have_at_least(&self, requested: usize) -> Result<()> {
        let remaining = self.remaining();
        if remaining < requested {
            let error = ErrorKind::UnexpectedEob { requested, remaining };
            Err(error.into())
        } else {
            Ok(())
        }
    }

    /// The implementation used by `peek_bytes_exact` and `read_bytes_exact`.
    /// It's implemented as a separate function so we can return a different lifetime than what the trait demands.
    ///
    /// The trait function requires we return a lifetime bound to `self`, whereas this function returns a lifetime
    /// bound to the underlying buffer (`'a`). Returning a narrower lifetime lets us mutate other fields of `self`.
    fn peek_bytes_exact_impl<const N: usize>(&self) -> Result<&'a [u8; N]> {
        let bytes = self.peek_byte_slice_exact_impl(N)?;

        // SAFETY: `peek_byte_slice_exact_impl` is guaranteed to return exactly 'N' bytes, which means it's safe to
        // convert, since `&[u8; N]` has the same layout as an `&[u8]` over 'N' bytes.
        unsafe {
            debug_assert_eq!(bytes.len(), N);
            Ok(bytes.try_into().unwrap_unchecked())
        }
    }

    /// The implementation used by `peek_byte_slice_exact` and `read_byte_slice_exact`.
    /// It's implemented as a separate function so we can return a different lifetime than what the trait demands.
    ///
    /// The trait function requires we return a lifetime bound to `self`, whereas this function returns a lifetime
    /// bound to the underlying buffer (`'a`). Returning a narrower lifetime lets us mutate other fields of `self`.
    fn peek_byte_slice_exact_impl(&self, count: usize) -> Result<&'a [u8]> {
        self.does_buffer_have_at_least(count)?;

        // SAFETY: the necessary bounds checking is performed by the above function call.
        unsafe {
            let end = self.pos + count;
            debug_assert!(self.buffer.get(self.pos..end).is_some());
            Ok(self.buffer.get_unchecked(self.pos..end))
        }
    }
}

impl InputSource for SliceInputSource<'_> {
    fn remaining(&self) -> usize {
        self.buffer.len() - self.pos
    }

    fn peek_byte(&mut self) -> Result<u8> {
        self.does_buffer_have_at_least(1)?;

        // SAFETY: the necessary bounds checking is performed by the above function call.
        unsafe {
            debug_assert!(self.buffer.get(self.pos).is_some());
            Ok(*self.buffer.get_unchecked(self.pos))
        }
    }

    fn read_byte(&mut self) -> Result<u8> {
        let byte = self.peek_byte()?;
        self.pos += 1;
        Ok(byte)
    }

    fn peek_bytes_exact<const N: usize>(&mut self) -> Result<&[u8; N]> {
        self.peek_bytes_exact_impl()
    }

    fn read_bytes_exact<const N: usize>(&mut self) -> Result<&[u8; N]> {
        let bytes = self.peek_bytes_exact_impl()?;
        self.pos += N;
        Ok(bytes)
    }

    fn peek_byte_slice_exact(&mut self, count: usize) -> Result<&[u8]> {
        self.peek_byte_slice_exact_impl(count)
    }

    fn read_byte_slice_exact(&mut self, count: usize) -> Result<&[u8]> {
        let byte_slice = self.peek_byte_slice_exact_impl(count)?;
        self.pos += count;
        Ok(byte_slice)
    }

    fn read_bytes_into_exact(&mut self, dst: &mut [u8]) -> Result<()> {
        let src = self.read_byte_slice_exact(dst.len())?;

        // SAFETY: `read_byte_slice_exact` is guaranteed to return exactly `dst.len()` bytes, so there is enough space
        // in `dst` to write these bytes, and we know the slices cannot overlap because `dst` is mutably borrowed,
        // which guarantees exclusive access.
        unsafe {
            debug_assert_eq!(src.len(), dst.len());
            core::ptr::copy_nonoverlapping(src.as_ptr(), dst.as_mut_ptr(), dst.len());
            Ok(())
        }
    }
}

impl<'a, T> From<&'a T> for SliceInputSource<'a>
where
    T: Borrow<[u8]> + ?Sized,
{
    /// Creates a new [`SliceInputSource`] that wraps the provided buffer.
    fn from(value: &'a T) -> Self {
        Self {
            buffer: value.borrow(),
            pos: 0,
        }
    }
}

// Allows users to create a [`Decoder`] directly from a slice,
// without needing to construct an intermediate [`SliceInputSource`].
#[cfg(feature = "slice2")]
impl<'a, T> From<T> for crate::decoder::Decoder<SliceInputSource<'a>>
where
    T: Into<SliceInputSource<'a>>,
{
    fn from(value: T) -> Self {
        crate::decoder::Decoder::new_with_inferred_encoding(value.into())
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
#[cfg(feature = "slice2")]
impl<'a, T> From<T> for crate::encoder::Encoder<SliceOutputTarget<'a>>
where
    T: Into<SliceOutputTarget<'a>>,
{
    fn from(value: T) -> Self {
        crate::encoder::Encoder::new_with_inferred_encoding(value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod slice_input_source {
        use super::*;

        /// Verifies that [`does_buffer_have_at_least`] returns the correct number of remaining bytes in the buffer
        /// when the remaining bytes number are greater than or equal to the number of requested bytes.
        #[test]
        fn does_buffer_has_at_least_returns_ok() {
            // Arrange
            let buffer = [115, 108, 105, 99, 101];
            let source = SliceInputSource::from(&buffer);

            // Act
            let result = source.does_buffer_have_at_least(5);

            // Assert
            assert!(result.is_ok());
        }

        /// Verifies that [`does_buffer_have_at_least`] returns an error when the remaining bytes number are less than
        /// the number of requested bytes.
        #[test]
        fn does_buffer_have_at_least_returns_error() {
            // Arrange
            let source = SliceInputSource::from(&[115, 108, 105, 99, 101]);

            // Act
            let result = source.does_buffer_have_at_least(6);

            // Assert
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().kind(), &ErrorKind::UnexpectedEob {
                requested: 6,
                remaining: 5,
            });
        }

        /// Verifies that [`peek_byte`] returns the correct byte from the buffer without consuming it.
        #[test]
        fn peek_byte_returns_correct_byte() {
            // Arrange
            let mut source = SliceInputSource::from(&[115, 108, 105, 99, 101]);

            // Act
            let result = source.peek_byte();

            // Assert
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 115);
            assert_eq!(source.pos, 0);
            assert_eq!(source.remaining(), 5);
        }

        /// Verifies that [`read_byte`] returns the correct byte from the buffer and consumes it.
        #[test]
        fn read_byte_returns_correct_byte() {
            // Arrange
            let mut source = SliceInputSource::from(&[115, 108, 105, 99, 101]);

            // Act
            let result = source.read_byte();

            // Assert
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 115);
            assert_eq!(source.pos, 1);
            assert_eq!(source.remaining(), 4);
        }

        /// Verifies that [`peek_bytes_exact`] returns the correct number of bytes from the buffer without consuming
        /// them.
        #[test]
        fn peek_bytes_exact_returns_correct_bytes() {
            // Arrange
            let mut source = SliceInputSource::from(&[115, 108, 105, 99, 101]);

            // Act
            let result = source.peek_bytes_exact::<3>();

            // Assert
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), &[115, 108, 105]);
            assert_eq!(source.pos, 0);
            assert_eq!(source.remaining(), 5);
        }

        /// Verifies that [`read_bytes_exact`] returns the correct number of bytes from the buffer and consumes them.
        #[test]
        fn read_bytes_exact_returns_correct_bytes() {
            // Arrange
            let mut source = SliceInputSource::from(&[115, 108, 105, 99, 101]);

            // Act
            let result = source.read_bytes_exact::<3>();

            // Assert
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), &[115, 108, 105]);
            assert_eq!(source.pos, 3);
            assert_eq!(source.remaining(), 2);
        }
    }

    mod slice_output_target {

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
            assert_eq!(result.unwrap_err().kind(), &ErrorKind::UnexpectedEob {
                requested: 6,
                remaining: 5,
            });
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
}
