
use crate::bit_sequence::BitSequenceWriter;
use crate::slice_encoder::*;
use crate::varints::{VarInt32, VarUInt32, VarInt62, VarUInt62};

#[derive(Debug)]
pub enum EncodeError {
    OutOfRange { value: i128, min: i128, max: i128, typename: &'static str },
}

#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;

#[cfg(feature = "std")]
use std::collections::HashMap;

pub trait SliceEncodable<E>
where
    Self: Sized,
    E: SliceEncoder,
{
    fn try_encode(self, encoder: &mut E) -> EncodeResult<()>;

    fn encode(self, encoder: &mut E) {
        default_error_handler(self.try_encode(encoder));
    }
}

pub trait SliceEncodableCollection<T, E>
where
    Self: Sized,
    E: SliceEncoder,
{
    fn try_encode_with_fn(self, encoder: &mut E, encode_element: EncodeFn<T, E>) -> EncodeResult<()>;

    fn encode_with_fn(self, encoder: &mut E, encode_element: EncodeFn<T, E>) {
        default_error_handler(self.try_encode_with_fn(encoder, encode_element));
    }
}

pub type EncodeFn<T, E> = fn(T, &mut E) -> EncodeResult<()>;

#[allow(dead_code)] // Seems like a clippy bug? This gets marked as unused even though it's obviously used.
#[inline(always)]
fn default_error_handler<T>(result: EncodeResult<T>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("failed to encode\n{error:?}"),
    }
}

// =============================================================================
// Fixed-length type implementations
// =============================================================================

// For primitive types, we implement `SliceEncodable` on the owned type, since it's conventional to
// pass these types 'by value'. But we also want `encode` to be available when they're borrowed too.
// This macro implements `SliceEncodable` on `&T` by delegating to the implementation for `T`.
macro_rules! implement_slice_encodable_for_borrowed_value_type {
    ($ty:ty, $encoder:ident$(: $($bound:tt)*)?) => {
        impl$(<$encoder: $($bound)*>)? SliceEncodable<$encoder> for &$ty {
            #[doc = concat!("Delegates to [", stringify!($ty), "::try_encode].")]                                       // TODO OPEN AN ISSUE, THIS GENERATED BOKRED LINKS [https://github.com/rust-lang/rust/issues/54172]
            #[inline(always)]
            fn try_encode(self, encoder: &mut $encoder) -> EncodeResult<()> {
                (*self).try_encode(encoder)
            }
        }
    }
}

impl<E: SliceEncoder> SliceEncodable<E> for bool {
    /// Encodes a value of `0` for `false` and a value of `1` for true, on a single byte.
    fn try_encode(self, encoder: &mut E) -> EncodeResult<()> {
        // In memory, bools are guaranteed to be `0` for false and `1` for `true`.
        encoder.write_byte(self as u8)
    }
}
implement_slice_encodable_for_borrowed_value_type!(bool, E: SliceEncoder);

impl<E: SliceEncoder> SliceEncodable<E> for u8 {
    /// Writes this byte directly into the encoder, as-is.
    fn try_encode(self, encoder: &mut E) -> EncodeResult<()> {
        encoder.write_byte(self)
    }
}
implement_slice_encodable_for_borrowed_value_type!(u8, E: SliceEncoder);

impl SliceEncodable<Slice2Encoder> for i8 {
    /// Encodes this i8 as a single byte, in two's component form.
    fn try_encode(self, encoder: &mut Slice2Encoder) -> EncodeResult<()> {
        // In memory, signed integers are guaranteed to use a two's complement representation.
        // Casting between i8 and u8 is no-op, and doesn't change this representation.
        encoder.write_byte(self as u8)
    }
}
implement_slice_encodable_for_borrowed_value_type!(i8, Slice2Encoder);

/// This macro is for implementing `SliceEncodable` on a numeric primitive type (and borrows of it).
/// Because all of these types have a `to_le_bytes` function that returns their representation in little endian.
///
/// Signed integers are always stored in two's compliment, and floating point numbers are always in IEEE 754 format.
macro_rules! implement_slice_encodable_for_primitive_numeric_type {
    ($ty:ty, $doc_text:literal, $encoder:ident$(: $($bound:tt)*)?) => {
        // Implement `SliceEncodable` for `ty`.
        impl$(<$encoder: $($bound)*>)? SliceEncodable<$encoder> for $ty {
            #[doc = $doc_text]
            fn try_encode(self, encoder: &mut $encoder) -> EncodeResult<()> {
                encoder.write_bytes(&self.to_le_bytes())
            }
        }

        // Implement `SliceEncodable` for `&ty`.
        implement_slice_encodable_for_borrowed_value_type!($ty, $encoder$(: $($bound)*)?);
    }
}

implement_slice_encodable_for_primitive_numeric_type!(
    u16,
    "Encodes this u16 on 2 bytes (little endian), as-is.",
    Slice2Encoder
);
implement_slice_encodable_for_primitive_numeric_type!(
    i16,
    "Encodes this i16 on 2 bytes (little endian), in two's complement form.",
    E: SliceEncoder
);
implement_slice_encodable_for_primitive_numeric_type!(
    u32,
    "Encodes this u32 on 4 bytes (little endian), as-is.",
    Slice2Encoder
);
implement_slice_encodable_for_primitive_numeric_type!(
    i32,
    "Encodes this i32 on 4 bytes (little endian), in two's complement form.",
    E: SliceEncoder
);
implement_slice_encodable_for_primitive_numeric_type!(
    u64,
    "Encodes this u64 on 8 bytes (little endian), as-is.",
    Slice2Encoder
);
implement_slice_encodable_for_primitive_numeric_type!(
    i64,
    "Encodes this i64 on 8 bytes (little endian), in two's complement form.",
    E: SliceEncoder
);
implement_slice_encodable_for_primitive_numeric_type!(
    f32,
    "Encodes this f32 on 4 bytes (little endian), using the \"binary32\" representation defined in IEEE 754-2008",
);
implement_slice_encodable_for_primitive_numeric_type!(
    f64,
    "Encodes this f64 on 8 bytes (little endian), using the \"binary64\" representation defined in IEEE 754-2008",
);

// =============================================================================
// Variable-length integer type implementations
// =============================================================================

fn compute_varint_size_prefix(preshifted_value: i64) -> i64 {
    // Integers must be the same type to be comparable, so we store pre-cast constants to de-clutter the comparisons.
    const I8_MAX: i64 = i8::MAX as i64;
    const I8_MIN: i64 = i8::MIN as i64;
    const I16_MAX: i64 = i16::MAX as i64;
    const I16_MIN: i64 = i16::MIN as i64;
    const I32_MAX: i64 = i32::MAX as i64;
    const I32_MIN: i64 = i32::MIN as i64;
    const I64_MAX: i64 = i64::MAX;
    const I64_MIN: i64 = i64::MIN;

    match preshifted_value {
        I8_MIN  ..= I8_MAX  => 0b00,  // 0
        I16_MIN ..= I16_MAX => 0b01,  // 1
        I32_MIN ..= I32_MAX => 0b10,  // 2
        I64_MIN ..= I64_MAX => 0b11,  // 3
    }
}

fn compute_varuint_size_prefix(preshifted_value: u64) -> u64 {
    // Integers must be the same type to be comparable, so we store pre-cast constants to de-clutter the comparisons.
    const U8_MAX: u64 = u8::MAX as u64;
    const U16_MAX: u64 = u16::MAX as u64;
    const U32_MAX: u64 = u32::MAX as u64;
    const U64_MAX: u64 = u64::MAX;

    match preshifted_value {
        ..= U8_MAX  => 0b00,  // 0
        ..= U16_MAX => 0b01,  // 1
        ..= U32_MAX => 0b10,  // 2
        ..= U64_MAX => 0b11,  // 3
    }
}

macro_rules! encode_variable_integer {
    ($value:ident, $encoder:ident, $compute_size_prefix:ident) => {{
        // Shift the value over by 2 bits to reserve room for the size prefix.
        $value <<= 2;

        // Calculate the minimum necessary size prefix, and OR it into the first 2 bits of value (which we reserved).
        let size_prefix = $compute_size_prefix($value);
        $value |= size_prefix;

        // Only encode the first `n` bytes of the value, where `n` is 2^size_prefix, so either 1, 2, 4, or 8 bytes.
        let encoded_length = usize::pow(2, size_prefix as u32);
        $encoder.write_bytes(&$value.to_le_bytes()[..encoded_length])
    }}
}

impl SliceEncodable for VarInt32 {
    fn try_encode(self, encoder: &mut SliceEncoder) -> EncodeResult<()> {
        let mut value = *self as i64;
        encode_variable_integer!(value, encoder, compute_varint_size_prefix)
    }
}

impl SliceEncodable for VarUInt32 {
    fn try_encode(self, encoder: &mut SliceEncoder) -> EncodeResult<()> {
        let mut value = *self as u64;
        encode_variable_integer!(value, encoder, compute_varuint_size_prefix)
    }
}

impl SliceEncodable for VarInt62 {
    fn try_encode(self, encoder: &mut SliceEncoder) -> EncodeResult<()> {
        if self < Self::MIN || self > Self::MAX {
            return Err(EncodeError::OutOfRange {
                value: *self as i128,
                min: *Self::MIN as i128,
                max: *Self::MAX as i128,
                typename: "varint62",
            })
        }

        let mut value = *self;
        encode_variable_integer!(value, encoder, compute_varint_size_prefix)
    }
}

impl SliceEncodable for VarUInt62 {
    fn try_encode(self, encoder: &mut SliceEncoder) -> EncodeResult<()> {
        // We don't check the `MIN` because it's 0, and it's impossible for this to hold a negative integer.
        if self > Self::MAX {
            return Err(EncodeError::OutOfRange {
                value: *self as i128,
                min: *Self::MIN as i128,
                max: *Self::MAX as i128,
                typename: "varuint62",
            })
        }

        let mut value = *self;
        encode_variable_integer!(value, encoder, compute_varuint_size_prefix)
    }
}

// =============================================================================
// Sequence type implementations
// =============================================================================

impl SliceEncodable for &str {
    /// Encodes this string by writing its length (encoded as a [varuint62](VarUInt62)),
    /// followed by its content (encoded in UTF-8 on `length` many bytes).
    fn try_encode(self, encoder: &mut SliceEncoder) -> EncodeResult<()> {
        // Note that this will silently wrap for strings longer than `u64::MAX`, this is practically impossible though.
        VarUInt62(self.len() as u64).try_encode(encoder)?;

        // Strings are always stored as UTF-8 in memory.
        encoder.write_bytes(self.as_bytes())
    }
}

impl<'a, T> SliceEncodableCollection<&'a T> for &'a [T] {
    /// Encodes this slice as a Slice sequence by writing its length (encoded as a [varuint62](VarUInt62)),
    /// followed by its elements, encoded in order using the provided `encode_element` function.
    fn try_encode_with_fn(self, encoder: &mut SliceEncoder, encode_element: EncodeFn<&'a T>) -> EncodeResult<()> {
        // Note that this will silently wrap for strings longer than `u64::MAX`, this is practically impossible though.
        VarUInt62(self.len() as u64).try_encode(encoder)?;

        for element in self {
            encode_element(element, encoder)?;
        }
        Ok(())
    }
}

/// Automatically implement `SliceEncodable` for slices whose elements are also `SliceEncodable`.
impl<'a, T> SliceEncodable for &'a [T]
    where &'a T: SliceEncodable,
{
    /// This delegates to [Self::try_encode_with_fn], using the element type's `try_encode` function.                   // TODO: This link is also broked.
    fn try_encode(self, encoder: &mut SliceEncoder) -> EncodeResult<()> {
        self.try_encode_with_fn(encoder, <&T>::try_encode)
    }
}

impl<'a, T> SliceEncodableCollection<&'a T> for &'a [Option<T>] {
    /// Encodes this slice of optionals as a Slice sequence by writing its length (encoded as a [varuint62](VarUInt62)),
    /// followed by a bit sequence, followed by its elements, encoded in order.
    ///
    /// If an element is `Some`, a `1` is written to the bit sequence, and the element's value is encoded using the
    /// provided `encode_element` function. Otherwise a `0` is written to the bit sequence and the element is skipped.
    fn try_encode_with_fn(self, encoder: &mut SliceEncoder, encode_element: EncodeFn<&'a T>) -> EncodeResult<()> {
        let length = self.len();

        // Note that this will silently wrap for strings longer than `u64::MAX`, this is practically impossible though.
        VarUInt62(length as u64).try_encode(encoder)?;

        // Reserve space in the encoder's buffer for the bit-sequence, and construct a BitSequenceWriter over it.
        let bit_sequence_size = length.div_ceil(8);
        let (bit_sequence_buffer, mut element_encoder) = encoder.reserve(bit_sequence_size)?;
        let mut bit_sequence_writer = BitSequenceWriter::new(bit_sequence_buffer);

        for element in self {
            bit_sequence_writer.write_bit(element.is_some());
            if let Some(value) = element {
                encode_element(value, &mut element_encoder)?;
            }
        }
        Ok(())
    }
}

/// Automatically implement `SliceEncodable` for slices of optionals whose elements are also `SliceEncodable`.
impl<'a, T> SliceEncodable for &'a [Option<T>]
    where &'a T: SliceEncodable,
{
    /// This delegates to [Self::try_encode_with_fn], using the element type's `try_encode` function.                   // TODO: This link is also broked.
    fn try_encode(self, encoder: &mut SliceEncoder) -> EncodeResult<()> {
        self.try_encode_with_fn(encoder, <&T>::try_encode)
    }
}

// =============================================================================
// Dictionary type implementations
// =============================================================================

/// This macro is for implementing `SliceEncodableCollection` on dictionary types, since their implementations are
/// identical. Unfortunately, Rust doesn't have a `Dictionary` trait that could be used for a blanket impl instead.
///
/// This macro also implements `SliceEncodable` for dictionaries of other `SliceEncodable` types.
macro_rules! implement_slice_encodable_for_dictionary_type {
    ($ty:ident, $feature:literal, $order_doc:literal) => {

#[cfg(feature = $feature)]
#[doc = concat!(
"Encodes this ", stringify!($ty), " as a Slice dictionary by writing its length (encoded as a [varuint62](VarUInt62)),
by its entries, encoded in ", $order_doc, ".
An entry is encoded by first encoding its key, followed by its value. This is equivalent to the following slice:
```slice
compact struct Entry
{
    key: K
    value: V
}
```"
)]
impl<'a, K, V> SliceEncodableCollection<(&'a K, &'a V)> for &'a $ty<K, V> {

    fn try_encode_with_fn(self, encoder: &mut SliceEncoder, encode_entry: EncodeFn<(&'a K, &'a V)>) -> EncodeResult<()> {
        // Note that this will silently wrap for strings longer than `u64::MAX`, this is practically impossible though.
        VarUInt62(self.len() as u64).try_encode(encoder)?;

        for entry in self {
            encode_entry(entry, encoder)?;
        }
        Ok(())
    }
}

#[cfg(feature = $feature)]
#[doc = concat!("Automatically implement `SliceEncodable` for ", stringify!($ty), "s whose keys and values are both `SliceEncodable`.")]
impl<'a, K, V> SliceEncodable for &'a $ty<K, V>
where
    &'a K: SliceEncodable,
    &'a V: SliceEncodable,
{
    /// This delegates to [Self::try_encode_with_fn], using the `try_encode` functions of the key and value types.      // TODO: This link is also broked.
    fn try_encode(self, encoder: &mut SliceEncoder) -> EncodeResult<()> {
        self.try_encode_with_fn(encoder, |(key, value), encoder| {
            key.try_encode(encoder)?;
            value.try_encode(encoder)
        })
    }
}

#[cfg(feature = $feature)]
#[doc = concat!("Automatically implement `SliceEncodable` for ", stringify!($ty), "s with optional values whose keys and values are both `SliceEncodable`.")]
impl<'a, K, V> SliceEncodable for &'a $ty<K, Option<V>>
where
    &'a K: SliceEncodable,
    &'a V: SliceEncodable,
{
    /// This delegates to [Self::try_encode_with_fn], using the `try_encode` functions of the key and value types.      // TODO: This link is also broked.
    fn try_encode(self, encoder: &mut SliceEncoder) -> EncodeResult<()> {
        self.try_encode_with_fn(encoder, |(key, value), encoder| {
            // Writing a bool is equivalent to a single element bit sequence; `1` is the value is `Some`, `0` otherwise.
            value.is_some().try_encode(encoder)?;
            key.try_encode(encoder)?;
            if let Some(v) = value {
                v.try_encode(encoder)?;
            }
            Ok(())
        })
    }
}

    }
}

implement_slice_encodable_for_dictionary_type!(HashMap, "std", "an arbitrary order");
implement_slice_encodable_for_dictionary_type!(BTreeMap, "alloc", "order");
