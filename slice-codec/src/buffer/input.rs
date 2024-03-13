// Copyright (c) ZeroC, Inc.

//! TODO maybe write a comment explaining this module?

use crate::{ErrorKind, Result};

use core::borrow::Borrow;
use core::convert::From;
use core::{debug_assert, debug_assert_eq};

/// A trait for types that can be read from by a [Slice decoder](crate::decoder::Decoder).
pub trait InputSource<'a> {
    /// Returns the next byte of input from this source, without consuming it.
    ///
    /// If there no more bytes available from this source, an [`ErrorKind::UnexpectedEob`] error is returned instead.
    fn peek_byte(&mut self) -> Result<u8>;

    /// Returns the next byte of input from this source, and advances past it (consuming it).
    ///
    /// If there no more bytes available from this source, an [`ErrorKind::UnexpectedEob`] error is returned instead.
    fn read_byte(&mut self) -> Result<u8>;

    // TODO these 4 functions need comments.
    // TODO remove any of these functions that don't end up being used anywhere.
    fn peek_bytes_exact<const N: usize>(&mut self) -> Result<&'a [u8; N]>;
    fn read_bytes_exact<const N: usize>(&mut self) -> Result<&'a [u8; N]>;

    fn peek_byte_slice_exact(&mut self, count: usize) -> Result<&'a [u8]>;
    fn read_byte_slice_exact(&mut self, count: usize) -> Result<&'a [u8]>;

    /// Reads bytes from this source into the provided buffer, and advances past them (consuming them).
    ///
    /// This function reads exactly `dest.len()`-many bytes, or if it's unable to, returns an error instead.
    /// If such an error occurs, no guarantees are made about how many bytes were read from the source, except that it
    /// is less than `dest.len()`.
    fn read_bytes_into_exact(&mut self, dest: &mut [u8]) -> Result<()>;
}

/// A wrapper around a `&[u8]` that implements [`InputSource`].
#[derive(Debug)]
pub struct SliceInputSource<'a> {
    /// The underlying buffer that this type wraps.
    source: &'a [u8],
    /// Tracks the current position in the buffer that is being read from.
    pos: usize,
}

impl<'a> SliceInputSource<'a> {
    fn ensure_source_has_at_least(&self, requested: usize) -> Result<()> {
        let remaining = self.source.len() - self.pos;
        if remaining < requested {
            let error = ErrorKind::UnexpectedEob { requested, remaining };
            Err(error.into())
        } else {
            Ok(())
        }
    }
}

impl<'a> InputSource<'a> for SliceInputSource<'a> {
    fn peek_byte(&mut self) -> Result<u8> {
        self.ensure_source_has_at_least(1)?;

        // SAFETY: the necessary bounds checking is performed by the above function call.
        unsafe {
            debug_assert!(self.source.get(self.pos).is_some());
            Ok(*self.source.get_unchecked(self.pos))
        }
    }

    fn read_byte(&mut self) -> Result<u8> {
        let byte = self.peek_byte()?;
        self.pos += 1;
        Ok(byte)
    }

    fn peek_bytes_exact<const N: usize>(&mut self) -> Result<&'a [u8; N]> {
        let bytes = self.peek_byte_slice_exact(N)?;

        // SAFETY: `peek_byte_slice_exact` is guaranteed to return exactly N bytes, so bytes is equivalent to `[u8; N]`.
        unsafe {
            debug_assert_eq!(bytes.len(), N);
            Ok(bytes.try_into().unwrap_unchecked())
        }
    }

    fn read_bytes_exact<const N: usize>(&mut self) -> Result<&'a [u8; N]> {
        let bytes = self.peek_bytes_exact()?;
        self.pos += N;
        Ok(bytes)
    }

    fn peek_byte_slice_exact(&mut self, count: usize) -> Result<&'a [u8]> {
        self.ensure_source_has_at_least(count)?;

        // SAFETY: the necessary bounds checking is performed by the above function call.
        unsafe {
            let end = self.pos + count;
            debug_assert!(self.source.get(self.pos..end).is_some());
            Ok(self.source.get_unchecked(self.pos..end))
        }
    }

    fn read_byte_slice_exact(&mut self, count: usize) -> Result<&'a [u8]> {
        let byte_slice = self.peek_byte_slice_exact(count)?;
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
    where T: Borrow<[u8]> + ?Sized
{
    /// Creates a new [`SliceInputSource`] that wraps the provided buffer.
    fn from(value: &'a T) -> Self {
        Self {
            source: value.borrow(),
            pos: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod slice_input_source {
        use super::*;
        use test_case::test_matrix;

        // TODO we need to write more comprehensive tests

        #[test]
        fn peeking_a_byte_does_not_advance_the_position() {
            // Arrange
            let buffer = [0, 1, 2, 3, 4, 5, 6, 7];
            let mut source = SliceInputSource::from(&buffer);
            assert_eq!(source.pos, 0);

            // Act
            let _ = source.peek_byte().unwrap();

            // Assert
            assert_eq!(source.pos, 0);
        }

        #[test_matrix([0, 1, 4, 7])]
        fn peeking_a_single_byte_returns_the_correct_value(bytes_to_skip: usize) {
            // Arrange
            let buffer = [0, 1, 2, 3, 4, 5, 6, 7];
            let mut source = SliceInputSource::from(&buffer);
            assert_eq!(source.read_byte_slice_exact(bytes_to_skip).unwrap(), &buffer[..bytes_to_skip]);
            assert_eq!(source.pos, bytes_to_skip);

            // Act
            let result = source.peek_byte().unwrap();

            // Assert
            assert_eq!(result, bytes_to_skip as u8);
            assert_eq!(source.pos, bytes_to_skip);
        }
    }
}
