// Copyright (c) ZeroC, Inc.

use slice_codec::buffer::slice::{SliceInputSource, SliceOutputTarget};
use slice_codec::buffer::{InputSource, OutputTarget};
use slice_codec::decode_from::DecodeFrom;
use slice_codec::decoder::Decoder;
use slice_codec::encode_into::EncodeInto;
use slice_codec::encoder::Encoder;

use core::fmt::Debug;

#[cfg(test)]
#[cfg(feature = "slice2")]
mod fixed_sized {

    use super::*;
    use slice_codec::slice2::Slice2;
    use test_case::test_case;

    // bool
    #[test_case(false, [0]; "false_bool")]
    #[test_case(true, [1]; "true_bool")]
    // uint8
    #[test_case(u8::MIN, [0]; "min_u8")]
    #[test_case(u8::MAX, [255]; "max_u8")]
    // int8
    #[test_case(i8::MIN, [128]; "min_i8")]
    #[test_case(0_i8, [0]; "zero_i8")]
    #[test_case(i8::MAX, [127]; "max_i8")]
    // uint16
    #[test_case(u16::MIN, [0, 0]; "min_u16")]
    #[test_case(u16::MAX, [255, 255]; "max_u16")]
    // int16
    #[test_case(i16::MIN, [0, 128]; "min_i16")]
    #[test_case(0_i16, [0, 0]; "zero_i16")]
    #[test_case(i16::MAX, [255, 127]; "max_i16")]
    // uint32
    #[test_case(u32::MIN, [0, 0, 0, 0]; "min_u32")]
    #[test_case(u32::MAX, [255, 255, 255, 255]; "max_u32")]
    // int32
    #[test_case(i32::MIN, [0, 0, 0, 128]; "min_i32")]
    #[test_case(0_i32, [0, 0, 0, 0]; "zero_i32")]
    #[test_case(i32::MAX, [255, 255, 255, 127]; "max_i32")]
    // uint64
    #[test_case(0_u64, [0, 0, 0, 0, 0, 0, 0, 0]; "zero_u64")]
    #[test_case(u64::MAX, [255, 255, 255, 255, 255, 255, 255, 255]; "max_u64")]
    // int64
    #[test_case(i64::MIN, [0, 0, 0, 0, 0, 0, 0, 128]; "min_i64")]
    #[test_case(0_i64, [0, 0, 0, 0, 0, 0, 0, 0]; "zero_i64")]
    #[test_case(i64::MAX, [255, 255, 255, 255, 255, 255, 255, 127]; "max_i64")]
    // f32
    #[test_case(f32::MIN, [255, 255, 127, 255]; "min_f32")]
    #[test_case(0_f32, [0, 0, 0, 0]; "zero_f32")]
    #[test_case(f32::MAX, [255, 255, 127, 127]; "max_f32")]
    // f64
    #[test_case(f64::MIN, [255, 255, 255, 255, 255, 255, 239, 255]; "min_f64")]
    #[test_case(0_f64, [0, 0, 0, 0, 0, 0, 0, 0]; "zero_f64")]
    #[test_case(f64::MAX, [255, 255, 255, 255, 255, 255, 239, 127]; "max_f64")]
    fn encoding_of<const N: usize, T>(value: T, expected: [u8; N])
    where
        T: EncodeInto<Slice2>,
    {
        // Arrange: create a buffer to encode the value into, and an encoder over that buffer.
        // Note: This test uses an array so it can run without needing the 'alloc' feature.
        let mut buffer = [0; N];
        let output_target = SliceOutputTarget::from(&mut buffer);
        let mut encoder = Encoder::new(output_target);

        // Act: encode the value, and ensure it succeeds.
        encoder.encode(value).expect("failed to encode");

        // Assert: ensure the buffer matches the expected bytes, and any remaining bytes are zeroed.
        let remaining_encoder_bytes = encoder.remaining();
        assert!(
            buffer.starts_with(&expected),
            "expected = {expected:?}\nactual   = {buffer:?}(ignore trailing 0's)"
        );
        let remaining = &buffer[expected.len()..];
        assert!(
            remaining.into_iter().all(|&x| x == 0),
            "remaining bytes weren't zeroed: {remaining:?}"
        );
        // Also make sure that the encoder used the expected number of bytes.
        assert_eq!(remaining_encoder_bytes, remaining.len());
    }

    // bool
    #[test_case(false, [0]; "false_bool")]
    #[test_case(true, [1]; "true_bool")]
    // uint8
    #[test_case(u8::MIN, [0]; "min_u8")]
    #[test_case(u8::MAX, [255]; "max_u8")]
    // int8
    #[test_case(i8::MIN, [128]; "min_i8")]
    #[test_case(0_i8, [0]; "zero_i8")]
    #[test_case(i8::MAX, [127]; "max_i8")]
    // uint16
    #[test_case(u16::MIN, [0, 0]; "min_u16")]
    #[test_case(u16::MAX, [255, 255]; "max_u16")]
    // int16
    #[test_case(i16::MIN, [0, 128]; "min_i16")]
    #[test_case(0_i16, [0, 0]; "zero_i16")]
    #[test_case(i16::MAX, [255, 127]; "max_i16")]
    // uint32
    #[test_case(u32::MIN, [0, 0, 0, 0]; "min_u32")]
    #[test_case(u32::MAX, [255, 255, 255, 255]; "max_u32")]
    // int32
    #[test_case(i32::MIN, [0, 0, 0, 128]; "min_i32")]
    #[test_case(0_i32, [0, 0, 0, 0]; "zero_i32")]
    #[test_case(i32::MAX, [255, 255, 255, 127]; "max_i32")]
    // uint64
    #[test_case(0_u64, [0, 0, 0, 0, 0, 0, 0, 0]; "zero_u64")]
    #[test_case(u64::MAX, [255, 255, 255, 255, 255, 255, 255, 255]; "max_u64")]
    // int64
    #[test_case(i64::MIN, [0, 0, 0, 0, 0, 0, 0, 128]; "min_i64")]
    #[test_case(0_i64, [0, 0, 0, 0, 0, 0, 0, 0]; "zero_i64")]
    #[test_case(i64::MAX, [255, 255, 255, 255, 255, 255, 255, 127]; "max_i64")]
    // f32
    #[test_case(f32::MIN, [255, 255, 127, 255]; "min_f32")]
    #[test_case(0_f32, [0, 0, 0, 0]; "zero_f32")]
    #[test_case(f32::MAX, [255, 255, 127, 127]; "max_f32")]
    // f64
    #[test_case(f64::MIN, [255, 255, 255, 255, 255, 255, 239, 255]; "min_f64")]
    #[test_case(0_f64, [0, 0, 0, 0, 0, 0, 0, 0]; "zero_f64")]
    #[test_case(f64::MAX, [255, 255, 255, 255, 255, 255, 239, 127]; "max_f64")]
    fn decoding_of<const N: usize, T>(expected: T, bytes: [u8; N])
    where
        T: DecodeFrom<Slice2> + Debug + PartialEq,
    {
        // Arrange: create a decoder over the provided bytes.
        let input_source = SliceInputSource::from(&bytes);
        let mut decoder = Decoder::new(input_source);

        // Act: decode a value of the specified type.
        let decoded_value: T = decoder.decode().expect("failed to decode");

        // Assert: the decoded value matches the expected value, and the decoder used all the provided bytes.
        assert_eq!(decoded_value, expected);
        assert_eq!(decoder.remaining(), 0);
    }
}

#[cfg(test)]
#[cfg(feature = "slice2")]
mod variable_sized {
    use super::*;

    mod encoding_of {

        use super::*;

        use slice_codec::slice2::{VARINT62_MAX, VARINT62_MIN};
        use test_case::test_case;

        #[test_case(0_u32, &[0x0]; "min_u32_one_byte")]
        #[test_case(2_u32.pow(6) - 1, &[0xFC]; "max_u32_one_byte")]
        #[test_case(2_u64.pow(6) - 1, &[0xFC]; "max_u64_one_byte")]
        #[test_case(2_u32.pow(6), &[0x1, 0x1]; "min_u32_two_bytes")]
        #[test_case(2_u64.pow(6), &[0x1, 0x1]; "min_u64_two_bytes")]
        #[test_case(2_u32.pow(14) - 1, &[0xFD, 0xFF]; "max_u32_two_bytes")]
        #[test_case(2_u64.pow(14) - 1, &[0xFD, 0xFF]; "max_u64_two_bytes")]
        #[test_case(2_u32.pow(14), &[0x2, 0x0, 0x1, 0x0]; "min_u32_four_bytes")]
        #[test_case(2_u64.pow(14), &[0x2, 0x0, 0x1, 0x0]; "min_u64_four_bytes")]
        #[test_case(2_u32.pow(30) - 1, &[0xFE, 0xFF, 0xFF, 0xFF]; "max_u32_four_bytes")]
        #[test_case(2_u64.pow(30) - 1, &[0xFE, 0xFF, 0xFF, 0xFF]; "max_u64_four_bytes")]
        #[test_case(2_u64.pow(30), &[0x3, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0]; "min_u64_eight_bytes")]
        #[test_case(2_u64.pow(62) - 1, &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]; "max_u64_eight_bytes")]
        fn varuint<T: PartialEq + Debug>(value: T, expected: &[u8])
        where
            u64: From<T>,
        {
            // Arrange
            // Note: This test uses an array so it can run without needing the 'alloc' feature.
            // 8 bytes is enough to hold any varuint. The actual size will be less so we can slice away the trailing
            // zeros if needed.
            let mut buffer = [0; 8];
            let output_target = SliceOutputTarget::from(&mut buffer);
            let mut encoder = Encoder::new(output_target);

            // Act
            let result = encoder.encode_varuint(value);

            // Assert
            assert!(result.is_ok(), "Encoding failed with error: {:?}", result.err());
            assert_eq!(&buffer[0..expected.len()], expected);
        }

        #[test]
        fn varuint_out_of_range() {
            // Arrange
            // Note: This test uses an array so it can run without needing the 'alloc' feature.
            let mut buffer = [];
            let value = 2u64.pow(62);
            let output_target = SliceOutputTarget::from(&mut buffer);
            let mut encoder = Encoder::new(output_target);

            // Act
            let result = encoder.encode_varuint(value);

            // Assert
            assert!(result.is_err());
        }

        #[test_case(0_i32, &[0]; "min_i32_one_byte")]
        #[test_case(0_i64, &[0]; "min_i64_one_byte")]
        #[test_case(2_i32.pow(6) - 1, &[253]; "max_i32_one_byte")]
        #[test_case(2_i64.pow(6) - 1, &[253]; "max_i64_one_byte")]
        #[test_case(2_i32.pow(6), &[1, 1]; "min_i32_two_bytes")]
        #[test_case(2_i64.pow(6), &[1, 1]; "min_i64_two_bytes")]
        #[test_case(2_i64.pow(14) - 1, &[254, 255]; "max_i32_two_bytes")]
        #[test_case(2_i64.pow(14) - 1, &[254, 255]; "max_i64_two_bytes")]
        #[test_case(2_i32.pow(14), &[2, 0, 1, 0]; "min_i32_four_bytes")]
        #[test_case(2_i64.pow(14), &[2, 0, 1, 0]; "min_i64_four_bytes")]
        #[test_case(2_i32.pow(30) - 1, &[255, 255, 255, 255]; "max_i32_four_bytes")]
        #[test_case(2_i64.pow(30) - 1, &[255, 255, 255, 255]; "max_i64_four_bytes")]
        #[test_case(2_i64.pow(30), &[3, 0, 0, 0, 1, 0, 0, 0]; "min_i64_eight_bytes")]
        #[test_case(VARINT62_MAX, &[255, 255, 255, 255, 255, 255, 255, 127]; "max_i64_eight_bytes_max")]
        fn varint<T: PartialEq + Debug>(value: T, expected: &[u8])
        where
            i64: From<T>,
        {
            // Arrange
            // Note: This test uses an array so it can run without needing the 'alloc' feature.
            let mut buffer = [0; 8];
            let output_target = SliceOutputTarget::from(&mut buffer);
            let mut encoder = Encoder::new(output_target);

            // Act
            let result = encoder.encode_varint(value);

            // Assert
            assert!(result.is_ok(), "Encoding failed with error: {:?}", result.err());
            assert_eq!(&buffer[0..expected.len()], expected);
        }

        #[test_case(VARINT62_MAX + 1; "maximum out of range")]
        #[test_case(VARINT62_MIN - 1; "minimum out of range")]
        fn varint_out_of_range(value: i64) {
            // Arrange
            // Note: This test uses an array so it can run without needing the 'alloc' feature.
            let mut buffer = [];
            let output_target = SliceOutputTarget::from(&mut buffer);
            let mut encoder = Encoder::new(output_target);

            // Act
            let result = encoder.encode_varint(value);
            // Assert
            assert!(result.is_err());
        }

        #[test_case(""; "empty_string")]
        #[test_case("Lorem ipsum dolor sit amet, no explicari repudiare vis, an dicant legimus ponderum sit.";  "Lorem")]
        #[test_case("국민경제의 발전을 위한 중요정책의 수립에 관하여 대통령의 자문에 응하기 위하여 국민경제자문회의를 둘 수 있다"; "Korean")]
        #[test_case(
            "旅ロ京青利セムレ弱改フヨス波府かばぼ意送でぼ調掲察たス日西重ケアナ住橋ユムミク順待ふかんぼ人奨貯鏡すびそ"
        ; "Japanese")]
        #[test_case("😁😂😃😄😅😆😉😊😋😌😍😏😒😓😔😖")]
        #[cfg(feature = "alloc")]
        fn string(str: &str) {
            // Arrange
            let mut buffer = vec![];
            let output_target = slice_codec::buffer::vec::VecOutputTarget::from(&mut buffer);
            let mut encoder = Encoder::new(output_target);
            let utf8_byte_count = str.as_bytes().len();

            // Act
            encoder.encode(str).expect("failed to encode string");

            // Assert
            let written_bytes = buffer.as_slice();
            let utf8_bytes = &written_bytes[(written_bytes.len() - utf8_byte_count)..];

            assert_eq!(str, String::from_utf8(utf8_bytes.to_vec()).unwrap());
        }
    }

    mod decoding_of {
        use super::*;

        use test_case::test_case;

        #[test_case(&[0x0], 0_u32 ; "min_u32_one_byte")]
        #[test_case(&[0x0], 0_u64 ; "min_u64_one_byte")]
        #[test_case(&[0xFC], 63_u32 ; "max_u32_one_byte")]
        #[test_case(&[0xFC], 63_u64 ; "max_u64_one_byte")]
        #[test_case(&[0x1, 0x1], 64_u32 ; "min_u32_two_bytes")]
        #[test_case(&[0x1, 0x1], 64_u64 ; "min_u64_two_bytes")]
        #[test_case(&[0xFD, 0xFF], 16383_u32; "max_u32_two_bytes")]
        #[test_case(&[0xFD, 0xFF], 16383_u64; "max_u64_two_bytes")]
        #[test_case(&[0x2, 0x0, 0x1, 0x0], 16384_u32; "min_u32_four_bytes")]
        #[test_case(&[0x2, 0x0, 0x1, 0x0], 16384_u64; "min_u64_four_bytes")]
        #[test_case(&[0xFE, 0xFF, 0xFF, 0xFF], 1073741823_u32; "max_u32_four_bytes")]
        #[test_case(&[0xFE, 0xFF, 0xFF, 0xFF], 1073741823_u64; "max_u64_four_bytes")]
        #[test_case(&[0x3, 0x0, 0x0, 0x0, 0x1, 0x0, 0x0, 0x0], 1073741824_u64; "min_u64_eight_bytes")]
        #[test_case(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], 2u64.pow(62) - 1 ; "max_u64_eight_bytes")]
        fn varuint<T: PartialEq + Debug + TryFrom<u64>>(bytes: &[u8], expected: T) {
            // Arrange
            let input_source = SliceInputSource::from(bytes);
            let mut decoder = Decoder::new(input_source);

            // Act
            let result = decoder.decode_varuint::<T>();

            // Assert
            assert!(result.is_ok(), "Decoding failed with error: {:?}", result.err());
            assert_eq!(
                result.unwrap(),
                expected,
                "Decoding result did not match expected value."
            );
        }

        #[test_case(&[0x80], -32_i32; "min_i32_one_byte")]
        #[test_case(&[0x80], -32_i64; "min_i64_one_byte")]
        #[test_case(&[0xFC], -1_i32; "max_i32_one_byte")]
        #[test_case(&[0xFC], -1_i64; "max_i64_one_byte")]
        #[test_case(&[0x1, 0x80], -8192_i32; "min_i32_two_bytes")]
        #[test_case(&[0x1, 0x80], -8192_i64; "min_i64_two_bytes")]
        #[test_case(&[0xFD, 0x7F], 8191_i32; "max_i32_two_bytes")]
        #[test_case(&[0xFD, 0x7F], 8191_i64; "max_i64_two_bytes")]
        #[test_case(&[0x2, 0x0, 0x0, 0x80], -536870912_i32; "min_i32_four_bytes")]
        #[test_case(&[0x2, 0x0, 0x0, 0x80], -536870912_i64; "min_i64_four_bytes")]
        #[test_case(&[0xFE, 0xFF, 0xFF, 0x7F], 536870911_i32; "max_i32_four_bytes")]
        #[test_case(&[0xFE, 0xFF, 0xFF, 0x7F], 536870911_i64; "max_i64_four_bytes")]
        fn varint<T: PartialEq + Debug + TryFrom<i64>>(bytes: &[u8], expected: T) {
            // Arrange
            let input_source = SliceInputSource::from(bytes);
            let mut decoder = Decoder::new(input_source);

            // Act
            let result = decoder.decode_varint::<T>();

            // Assert
            assert!(result.is_ok(), "Decoding failed with error: {:?}", result.err());
            assert_eq!(
                result.unwrap(),
                expected,
                "Decoding result did not match expected value."
            );
        }

        #[test_case(""; "empty_string")]
        #[test_case("Lorem ipsum dolor sit amet, no explicari repudiare vis, an dicant legimus ponderum sit."; "Lorem")]
        #[test_case("국민경제의 발전을 위한 중요정책의 수립에 관하여 대통령의 자문에 응하기 위하여 국민경제자문회의를 둘 수 있다"; "Korean")]
        #[test_case(
            "旅ロ京青利セムレ弱改フヨス波府かばぼ意送でぼ調掲察たス日西重ケアナ住橋ユムミク順待ふかんぼ人奨貯鏡すびそ"
        ; "Japanese")]
        #[test_case("😁😂😃😄😅😆😉😊😋😌😍😏😒😓😔😖")]
        #[cfg(feature = "alloc")]
        fn string(str: &str) {
            // Arrange
            let mut buffer = vec![];
            let output_target = slice_codec::buffer::vec::VecOutputTarget::from(&mut buffer);
            let mut encoder = Encoder::new(output_target);
            encoder.encode(str).expect("failed to encode string");

            let buffer_two = buffer.clone();
            let input_source = SliceInputSource::from(&buffer_two);
            let mut decoder = Decoder::new(input_source);

            // Act
            let decoded = decoder.decode::<String>();

            // Assert
            assert!(decoded.is_ok());
            assert_eq!(decoded.unwrap(), str);
        }

        #[test]
        #[cfg(feature = "alloc")]
        fn non_utf8_string() {
            // Arrange
            let buffer = [0x08, 0xFD, 0xFF];
            let input_source = SliceInputSource::from(&buffer);
            let mut decoder = Decoder::new(input_source);

            // Act
            let decoded = decoder.decode::<String>();

            // Assert
            assert!(decoded.is_err());
            assert!(matches!(
                decoded.err().unwrap().kind(),
                slice_codec::ErrorKind::InvalidData(slice_codec::InvalidDataErrorKind::InvalidString(_))
            ));
        }
    }
}
