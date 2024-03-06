// Copyright (c) ZeroC, Inc.

use crate::try_encode::TryEncode;
use crate::{Encoding, Error, Result};

/// TODO
#[derive(Debug)]
pub struct Encoder<'a, E: Encoding> {
    /// Which version of the Slice encoding this encoder is using.
    _encoding: core::marker::PhantomData<E>,

    buffer_temp: core::marker::PhantomData<&'a ()>, // TODO
}

impl<'a, E: Encoding> Encoder<'a, E> {
    // TODO new function here?

    /// Attempts to encode a value of `T` into this encoder's buffer.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer: &mut [u8] = &mut [0; 5];
    /// let encoder = Encoder::new(buffer);
    ///
    /// assert!(encoder.try_encode(1701740006_i32).is_ok());
    /// assert!(encoder.try_encode(true).is_ok());
    ///
    /// assert_eq!(buffer, [230, 125, 110, 101, 1]);
    /// ```
    pub fn try_encode<T: TryEncode<E>>(&mut self, value: T) -> Result<()> {
        value.try_encode(self)
    }

    /// Writes the provided byte to this encoder's buffer, then advances this encoder's position by 1.
    ///
    /// If the encoder is at end-of-buffer, `Err` is returned instead.
    /// Encoders that use dynamically growable buffers (like [`Vec`](std::vec::Vec)) will never encounter this.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer: &mut [u8] = &mut [0; 3];
    /// let encoder = Encoder::new(buffer);
    ///
    /// // Write some bytes to the encoder's buffer, one at a time.
    /// assert!(encoder.write_byte(1).is_ok());
    /// assert!(encoder.write_byte(2).is_ok());
    /// assert!(encoder.write_byte(8).is_ok());
    /// assert_eq!(buffer, [1, 2, 8]);
    ///
    /// // Until end-of-buffer is reached (since we're using a non-growable buffer here).
    /// assert!(encoder.write_byte(0).is_err());
    /// ```
    pub fn write_byte(&mut self, data: u8) -> Result<()> {
        todo!() // TODO
    }

    /// Writes the provided bytes to this encoder's buffer, then advances this encoder's positions by `data.len`.
    ///
    /// If there are less than `data.len` many bytes available in the buffer, this returns `Err` instead.
    /// In this case, no bytes are written, and the encoder's position is not advanced.
    /// Encoders that use dynamically growable buffers (like [`Vec`](std::vec::Vec)) will never encounter this.
    ///
    /// # Examples
    ///
    /// ```
    /// let buffer: &mut [u8] = &mut [0; 6];
    /// let encoder = Encoder::new(buffer);
    ///
    /// // Write some bytes to the encoder's buffer.
    /// assert!(encoder.write_bytes_exact(&[1, 2]).is_ok());
    /// assert!(encoder.write_bytes_exact(&[3]).is_ok());
    /// assert!(encoder.write_bytes_exact(&[]).is_ok());
    /// assert!(encoder.write_bytes_exact(&[4]).is_ok());
    ///
    /// // `write_bytes_exact` returns an error if you try to write more bytes than are available...
    /// assert!(encoder.write_bytes(&[1; 10]).is_err());
    ///
    /// // ... but leaves the buffer unaffected, so any remaining bytes can still be written to.
    /// assert!(encoder.write_bytes_exact().is_ok());
    ///
    /// assert_eq!(buffer, [1, 2, 3, 4, 5, 6]);
    /// ```
    pub fn write_bytes_exact(&mut self, data: &[u8]) -> Result<()> {
        todo!() // TODO
    }

    /// TODO
    pub fn reserve(&mut self, count: usize) -> Result<&'a mut [u8]> {
        todo!() // TODO
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
