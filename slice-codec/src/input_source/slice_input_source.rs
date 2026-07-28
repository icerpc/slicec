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

    fn read_bytes_exact<const N: usize>(&mut self) -> Result<&[u8; N]> {
        let byte_slice = self.read_byte_slice_exact(N)?;

        // SAFETY: `read_byte_slice_exact` is guaranteed to return exactly 'N' bytes, which means it's safe to
        // convert, since `&[u8; N]` has the same layout as an `&[u8]` over 'N' bytes.
        let byte_array = unsafe {
            debug_assert_eq!(byte_slice.len(), N);
            byte_slice.try_into().unwrap_unchecked()
        };

        Ok(byte_array)
    }

    fn read_byte_slice_exact(&mut self, count: usize) -> Result<&[u8]> {
        self.does_buffer_have_at_least(count)?;

        // SAFETY: the necessary bounds checking is performed by the above function call.
        let byte_slice = unsafe {
            let end = self.pos + count;
            debug_assert!(self.buffer.get(self.pos..end).is_some());
            self.buffer.get_unchecked(self.pos..end)
        };
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
impl<'a, T> From<T> for crate::decoding::decoder::Decoder<SliceInputSource<'a>>
where
    T: Into<SliceInputSource<'a>>,
{
    fn from(value: T) -> Self {
        crate::decoding::decoder::Decoder::new(value.into())
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
        assert!(matches!(result.unwrap_err().kind(), ErrorKind::UnexpectedEob {
            requested: 6,
            remaining: 5
        }));
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
