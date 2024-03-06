// Copyright (c) ZeroC, Inc.

pub struct BitSequenceReader<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> BitSequenceReader<'a> {
    pub fn new(buffer: &'a [u8], length: usize) -> Self {
        BitSequenceReader {
            buffer,
            position: 0,
        }
    }

    // This will panic if it gets out of bounds!
    pub fn read_bit(&mut self) -> bool {
        debug_assert!(self.position < (self.buffer.len() * 8));

        let byte_index = self.position / 8;                                                        // The compiler is smart enough to use `shr 3` for
        let bit_index = self.position % 8;                                                         // both of these operations without `>> 3` and `& 0x7`.
        self.position += 1;

        (self.buffer[byte_index] & (1 << bit_index)) != 0
    }
}

pub struct BitSequenceWriter<'a> {
    buffer: &'a mut [u8],
    position: usize,
}

impl<'a> BitSequenceWriter<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        buffer.fill(0b00000000); // Zero the buffer.                                               // This gets compiled into a memset (the most efficient).

        BitSequenceWriter {
            buffer,
            position: 0,
        }
    }

    // This will panic if it gets out of bounds!
    pub fn write_bit(&mut self, value: bool) {
        debug_assert!(self.position < (self.buffer.len() * 8));

        // We only need to set bits if value is true, since we zeroed the buffer in `new`.
        if value {
            let byte_index = self.position / 8;                                                    // The compiler is smart enough to use `shr 3` for
            let bit_index = self.position % 8;                                                     // both of these operations without `>> 3` and `& 0x7`.
            self.buffer[byte_index] |= 1 << bit_index;
        }

        self.position += 1;
    }
}
