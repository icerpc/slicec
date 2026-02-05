// Copyright (c) ZeroC, Inc.

//! TODO maybe write a comment explaining this module?

pub mod slice;

#[cfg(feature = "alloc")]
pub mod vec;

use crate::Result;
use core::ops::Range;

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
    fn read_bytes_into_buffer(&mut self, dest: &mut [u8]) -> Result<()>;
}

/// A trait for types that can be written to by a [Slice encoder](crate::encoder::Encoder).
pub trait OutputTarget {
    /// Returns the number of unwritten bytes currently remaining in the target.
    ///
    /// Note: some implementations are capable of growing their underlying buffers as needed. For these implementations,
    /// this function returns how many bytes can be written _without needing to grow_ ie. "... currently remaining ...".
    /// For these types, it's generally safe to write more than `remaining()` bytes to the target, since it will simply
    /// grow as needed. But these writes can still fail if an allocation error occurs.
    fn remaining(&self) -> usize;

    /// Attempts to write the provided byte into this target.
    fn write_byte(&mut self, byte: u8) -> Result<()>;

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
#[derive(Debug, PartialEq)]
#[must_use]
pub struct Reservation(Range<usize>);

impl Reservation {
    /// Returns a [`Range`] corresponding to the byte positions held by this [`Reservation`].
    fn range(&self) -> Range<usize> {
        self.0.clone()
    }
}
