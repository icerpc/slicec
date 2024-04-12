// Copyright (c) ZeroC, Inc.

//! TODO maybe write a comment explaining this module?

use crate::{ErrorKind, Result};

use core::borrow::Borrow;
use core::{debug_assert, debug_assert_eq};

/// A trait for types that can be read from by a [Slice decoder](crate::decoder::Decoder).
pub trait InputSource {
    /// Returns the number of unread bytes currently remaining in the source.
    fn remaining(&self) -> usize;

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
    fn peek_bytes_exact<const N: usize>(&mut self) -> Result<&[u8; N]>;
    fn read_bytes_exact<const N: usize>(&mut self) -> Result<&[u8; N]>;

    fn peek_byte_slice_exact(&mut self, count: usize) -> Result<&[u8]>;
    fn read_byte_slice_exact(&mut self, count: usize) -> Result<&[u8]>;

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
    where T: Borrow<[u8]> + ?Sized
{
    /// Creates a new [`SliceInputSource`] that wraps the provided buffer.
    fn from(value: &'a T) -> Self {
        Self {
            buffer: value.borrow(),
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
            let mut input = SliceInputSource::from(&buffer);
            assert_eq!(input.pos, 0);

            // Act
            let _ = input.peek_byte().unwrap();

            // Assert
            assert_eq!(input.pos, 0);
        }

        #[test_matrix([0, 1, 4, 7])]
        fn peeking_a_single_byte_returns_the_correct_value(bytes_to_skip: usize) {
            // Arrange
            let buffer = [0, 1, 2, 3, 4, 5, 6, 7];
            let mut input = SliceInputSource::from(&buffer);
            assert_eq!(input.read_byte_slice_exact(bytes_to_skip).unwrap(), &buffer[..bytes_to_skip]);
            assert_eq!(input.pos, bytes_to_skip);

            // Act
            let result = input.peek_byte().unwrap();

            // Assert
            assert_eq!(result, bytes_to_skip as u8);
            assert_eq!(input.pos, bytes_to_skip);
        }
    }
}
