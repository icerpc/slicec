// Copyright (c) ZeroC, Inc.

use super::{Slice2, Slice2Encoder};
use crate::buf_io::bit_sequence::BitSequenceWriter;
use crate::encoder::Encoder;
use crate::encoding::{implement_try_encode_for_primitive_numeric_type, implement_try_encode_on_borrowed_type};
use crate::try_encode::{EncodeFn, TryEncode, TryEncodeCollection};
use crate::{Encoding, Error, Result};

// We only support `BTreeMap` if the `alloc` crate is available through the `alloc` feature flag.
#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

// We only support `HashMap` if the standard library is available through the `std` feature flag.
#[cfg(feature = "std")]
use std::collections::HashMap;

// =============================================================================
// Fixed-length type implementations
// =============================================================================

impl TryEncode<Slice2> for i8 {
    fn try_encode(self, encoder: &mut Slice2Encoder) -> Result<()> {
        encoder.write_byte(self as u8)
    }
}
implement_try_encode_on_borrowed_type!(i8, Slice2);

implement_try_encode_for_primitive_numeric_type!(u16, "TODO", Slice2);
implement_try_encode_for_primitive_numeric_type!(u32, "TODO", Slice2);
implement_try_encode_for_primitive_numeric_type!(u64, "TODO", Slice2);

// =============================================================================
// Variable-length integer type implementations
// =============================================================================

impl Slice2Encoder<'_> {

}

#[cfg(test)]
mod tests {
    // TODO
}
