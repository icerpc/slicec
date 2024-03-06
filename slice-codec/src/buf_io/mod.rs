// Copyright (c) ZeroC, Inc.

// TODO THIS WHOLE MODULE IS A WORK IN PROGRESS!!

pub mod bit_sequence;

pub enum Error {
    UnexpectedEof,
    InsufficientSpace {
        requested: usize,
        remaining: usize,
    }
}
pub type Result<T> = core::result::Result<T, Error>;






pub struct BufferedInput<'a> {
    source: InputSource<'a>,
    buffer: &'a [u8],
    pos: usize,
}

impl BufferedInput<'_> {
    pub fn peek_byte(&mut self) -> Result<&u8> {
        if self.pos < self.buffer.len() {
            Ok(&self.buffer[self.pos - 1])
        } else {
            Err(Error::UnexpectedEof)
        }
    }

    pub fn read_byte(&mut self) -> Result<&u8> {
        if self.pos < self.buffer.len() {
            self.pos += 1;
            Ok(&self.buffer[self.pos - 1])
        } else {
            Err(Error::UnexpectedEof)
        }
    }

    pub fn peek_bytes_exact<const N: usize>(&mut self) -> Result<&[u8; N]> {
        let bytes = self.peek_byte_slice_exact(N)?;
        // SAFETY: unwrapping is safe because `peek_byte_slice_exact` guarantees `bytes` will have the correct length.
        Ok(bytes.try_into().unwrap())
    }

    pub fn read_bytes_exact<const N: usize>(&mut self) -> Result<&[u8; N]> {
        let bytes = self.read_byte_slice_exact(N)?;
        // SAFETY: unwrapping is safe because `read_byte_slice_exact` guarantees `bytes` will have the correct length.
        Ok(bytes.try_into().unwrap())
    }

    pub fn peek_byte_slice_exact(&mut self, count: usize) -> Result<&[u8]> {
        let end = self.pos + count;
        if end < self.buffer.len() {
            // SAFETY: The necessary bound checks are performed by the above if statement.
            unsafe {
                Ok(&self.buffer.get_unchecked(self.pos..end))
            }
        } else {
            Err(Error::InsufficientSpace {
                requested: count,
                remaining: self.buffer.len() - self.pos,
            })
        }
    }

    pub fn read_byte_slice_exact(&mut self, count: usize) -> Result<&[u8]> {
        let end = self.pos + count;
        if end < self.buffer.len() {
            // SAFETY: The necessary bound checks are performed by the above if statement.
            unsafe {
                let slice = &self.buffer.get_unchecked(self.pos..end);
                self.pos = end;
                Ok(slice)
            }
        } else {
            Err(Error::InsufficientSpace {
                requested: count,
                remaining: self.buffer.len() - self.pos,
            })
        }
    }

    pub fn reserve(&mut self, count: usize) -> Result<(&[u8], Self)> {
        let split_pos = self.pos + count;

        if split_pos < self.buffer.len() {
            // SAFETY: The necessary bound checks are performed by the above if statement.
            // TODO: simplify after 'https://github.com/rust-lang/rust/issues/76014' is stabilized.
            let (reserved, remaining) = unsafe {
                (self.buffer.get_unchecked(..split_pos), self.buffer.get_unchecked(split_pos..))
            };
            let child = match &self.source {
                InputSource::Slice(_) => BufferedInput::from(remaining),
            };

            Ok((reserved, child))
        } else {
            Err(Error::InsufficientSpace {
                requested: count,
                remaining: self.buffer.len() - self.pos,
            })
        }
    }
}

impl<'a> From<&'a [u8]> for BufferedInput<'a> {
    fn from(slice: &'a [u8]) -> Self {
        BufferedInput {
            source: InputSource::Slice(slice),
            buffer: slice,
            pos : 0,
        }
    }
}

pub enum InputSource<'a> {
    Slice(&'a [u8]),
}










pub enum OutputTarget<'a> {
    Slice(&'a mut [u8])

}