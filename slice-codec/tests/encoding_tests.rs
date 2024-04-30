// Copyright (c) ZeroC, Inc.

use slice_codec::buffer::slice::{SliceInputSource, SliceOutputTarget};
use slice_codec::buffer::{InputSource, OutputTarget};
use slice_codec::decode_from::DecodeFrom;
use slice_codec::decoder::Decoder;
use slice_codec::encode_into::EncodeInto;
use slice_codec::encoder::Encoder;

use core::fmt::Debug;

use test_case::test_case;

/// TODO
macro_rules! generate_test_cases_for {
    ($encoding:ty, $($value:expr, $bytes:expr, $name:literal,)*) => {
        $(#[test_case($value, $bytes; $name)])*
        fn encoding_of<const N: usize, T>(value: T, expected: [u8; N])
            where T: EncodeInto<$encoding>,
        {
            // Arrange: create a buffer to encode the value into, and an encoder over that buffer.
            let mut buffer = [0; 256];
            let output_target = SliceOutputTarget::from(&mut buffer);
            let mut encoder: Encoder<_, $encoding> = Encoder::new_with_inferred_encoding(output_target);

            // Act: encode the value, and ensure it succeeds.
            encoder.encode(value).expect("failed to encode");

            // Assert: ensure the buffer matches the expected bytes, and any remaining bytes are zeroed.
            let remaining_encoder_bytes = encoder.remaining();
            assert!(buffer.starts_with(&expected), "expected = {expected:?}\nactual   = {buffer:?}(ignore trailing 0's)");
            let remaining = &buffer[expected.len()..];
            assert!(remaining.into_iter().all(|&x| x == 0), "remaining bytes weren't zeroed: {remaining:?}");
            // Also make sure that the encoder used the expected number of bytes.
            assert_eq!(remaining_encoder_bytes, remaining.len());
        }

        $(#[test_case($bytes, $value; $name)])*
        fn decoding_of<const N: usize, T>(bytes: [u8; N], expected: T)
            where T: DecodeFrom<$encoding> + Debug + PartialEq,
        {
            // Arrange: create a decoder over the provided bytes.
            let input_source = SliceInputSource::from(&bytes);
            let mut decoder: Decoder<_, $encoding> = Decoder::new_with_inferred_encoding(input_source);

            // Act: decode a value of the specified type.
            let decoded_value: T = decoder.decode().expect("failed to decode");

            // Assert: the decoded value matches the expected value, and the decoder used all the provided bytes.
            assert_eq!(decoded_value, expected);
            assert_eq!(decoder.remaining(), 0);
        }
    };
}

#[rustfmt::skip] // This macro is organized so that each line represents 1 test case. 'rustfmt' break this obviously.
#[cfg(feature = "slice2")]
generate_test_cases_for!(slice_codec::slice2::Slice2,
    // Integral types
    // bool
    false, [0], "false_bool",
    true, [1], "true_bool",

    // uint8
    u8::MIN, [0], "min_u8",
    u8::MAX, [255], "max_u8",

    // int8
    i8::MIN, [128], "min_i8",
    0_i8, [0], "zero_i8",
    i8::MAX, [127], "max_i8",

    // uint16
    u16::MIN, [0, 0], "min_u16",
    u16::MAX, [255, 255], "max_u16",

    // int16
    i16::MIN, [0, 128], "min_i16",
    0_i16, [0, 0], "zero_i16",
    i16::MAX, [255, 127], "max_i16",

    // uint32
    u32::MIN, [0, 0, 0, 0], "min_u32",
    u32::MAX, [255, 255, 255, 255], "max_u32",

    // int32
    i32::MIN, [0, 0, 0, 128], "min_i32",
    0_i32, [0, 0, 0, 0], "zero_i32",
    i32::MAX, [255, 255, 255, 127], "max_i32",

    // uint64
    0_u64, [0, 0, 0, 0, 0, 0, 0, 0], "zero_u64",
    u64::MAX, [255, 255, 255, 255, 255, 255, 255, 255], "max_u64",

    // int64
    i64::MIN, [0, 0, 0, 0, 0, 0, 0, 128], "min_i64",
    0_i64, [0, 0, 0, 0, 0, 0, 0, 0], "zero_i64",
    i64::MAX, [255, 255, 255, 255, 255, 255, 255, 127], "max_i64",

    // Floating-point types
    // f32
    f32::MIN, [255, 255, 127, 255], "min_f32",
    0_f32, [0, 0, 0, 0], "zero_f32",
    f32::MAX, [255, 255, 127, 127], "max_f32",

    // f64
    f64::MIN, [255, 255, 255, 255, 255, 255, 239, 255], "min_f64",
    0_f64, [0, 0, 0, 0, 0, 0, 0, 0], "zero_f64",
    f64::MAX, [255, 255, 255, 255, 255, 255, 239, 127], "max_f64",
);
