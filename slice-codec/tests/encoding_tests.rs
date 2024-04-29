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
        fn test_encoding_of<const N: usize, T>(value: T, expected: [u8; N])
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
        fn test_decoding_of<const N: usize, T>(bytes: [u8; N], expected: T)
            where T: DecodeFrom<$encoding> + Debug + Eq,
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
    false, [0], "false_bool",
    true, [1], "true_bool",
);
