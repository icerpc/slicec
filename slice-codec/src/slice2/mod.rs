// Copyright (c) ZeroC, Inc.

// These modules are private because they don't export any types, just implementations.
mod decoding;
mod encoding;

use crate::Encoding;

/// The smallest value that can be represented as a `varint32`.
pub const VARINT32_MIN: i32 = i32::MIN;
/// The largest value that can be represented as a `varint32`.
pub const VARINT32_MAX: i32 = i32::MAX;
/// The smallest value that can be represented as a `varuint32`.
pub const VARUINT32_MIN: u32 = u32::MIN;
/// The largest value that can be represented as a `varuint32`.
pub const VARUINT32_MAX: u32 = u32::MAX;
/// The smallest value that can be represented as a `varint62`.
pub const VARINT62_MIN: i64 = i64::MIN >> 2;
/// The largest value that can be represented as a `varint62`.
pub const VARINT62_MAX: i64 = i64::MAX >> 2;
/// The smallest value that can be represented as a `varuint62`.
pub const VARUINT62_MIN: u64 = u64::MIN >> 2;
/// The largest value that can be represented as a `varuint62`.
pub const VARUINT62_MAX: u64 = u64::MAX >> 2;

/// Version 2 of the (Slice encoding)[<https://docs.icerpc.dev/slice2/encoding/overview>].
/// This is the default encoding version used by this crate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Slice2;

impl Encoding for Slice2 {}
