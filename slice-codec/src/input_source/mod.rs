// Copyright (c) ZeroC, Inc.

//! TODO maybe write a comment explaining this module?

mod slice_input_source;
pub use slice_input_source::*;

use crate::Result;

/// A trait for types that can be read from by a [Slice decoder](crate::decoder::Decoder).
pub trait InputSource {
    /// Returns the number of unread bytes currently remaining in the source.
    fn remaining(&self) -> usize;

    /// Returns the next byte available from this source without consuming it.
    ///
    /// If there are no more bytes available from this source, an [`UnexpectedEob`] error is returned instead.
    ///
    /// [`UnexpectedEob`]: crate::ErrorKind::UnexpectedEob
    fn peek_byte(&mut self) -> Result<u8>;

    /// Returns the next byte available from this source, and advances past it (consuming it).
    ///
    /// If there are no more bytes available from this source, an [`UnexpectedEob`] error is returned instead.
    ///
    /// [`UnexpectedEob`]: crate::ErrorKind::UnexpectedEob
    fn read_byte(&mut self) -> Result<u8>;

    // TODO remove these functions after adding `advance_by` and the low-level required API functions.
    fn read_bytes_exact<const N: usize>(&mut self) -> Result<&[u8; N]>;
    fn read_byte_slice_exact(&mut self, count: usize) -> Result<&[u8]>;

    /// Reads bytes from this source into the provided buffer, and advances past them (consuming them).
    ///
    /// This function reads exactly `dest.len()`-many bytes, or if it's unable to, returns an error instead.
    /// If such an error occurs, no guarantees are made about how many bytes were read from the source, except that it
    /// is less than `dest.len()`.
    fn read_bytes_into_exact(&mut self, dest: &mut [u8]) -> Result<()>;
}
