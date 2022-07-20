// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice1 {

    use crate::helpers::parsing_helpers::parse_for_errors;
    use crate::{assert_errors, assert_errors_new};
    use slice::errors::*;
    use test_case::test_case;

    /// Verifies that if Slice1 is used with unsupported types (int8, uint16, uint32, varint32,
    /// varuint32, uint64, varint62, and varuint62) that the compiler will produce the relevant not
    /// supported errors.
    #[test_case("int8"; "int8")]
    #[test_case("uint16"; "uint16")]
    #[test_case("uint32"; "uint32")]
    #[test_case("varint32"; "varint32")]
    #[test_case("varuint32"; "varuint32")]
    #[test_case("uint64"; "uint64")]
    #[test_case("varint62"; "varint62")]
    #[test_case("varuint62"; "varuint62")]
    fn unsupported_types_fail(value: &str) {
        // Test setup
        let slice = &format!(
            "
                encoding = 1;
                module Test;
                compact struct S
                {{
                    v: {value},
                }}
            ",
            value = value,
        );
        let expected: [&dyn ErrorType; 2] = [
            &RuleKind::from(InvalidEncodingKind::UnsupportedType {
                type_string: value.to_owned(),
                encoding: "1".to_owned(),
            }),
            &Note::new("file encoding was set to Slice1 here:"),
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors_new!(error_reporter, expected);
    }

    /// Verifies that valid Slice1 types (bool, uint8, int16, int32, int64, float32, float64,
    /// string, and  AnyClass) will not produce any compiler errors.
    #[test_case("bool"; "bool")]
    #[test_case("uint8"; "uint8")]
    #[test_case("int16"; "int16")]
    #[test_case("int32"; "int32")]
    #[test_case("int64"; "int64")]
    #[test_case("float32"; "float32")]
    #[test_case("float64"; "float64")]
    #[test_case("string"; "string")]
    #[test_case("AnyClass"; "AnyClass")]
    fn supported_types_succeed(value: &str) {
        // Arrange
        let slice = &format!(
            "
                encoding = 1;
                module Test;
                compact struct S
                {{
                    v: {value},
                }}
            ",
            value = value,
        );

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter);
    }
}

mod slice2 {

    use crate::helpers::parsing_helpers::parse_for_errors;
    use crate::{assert_errors, assert_errors_new};
    use slice::errors::*;
    use test_case::test_case;

    /// Verifies that if Slice2 is used with unsupported types (AnyClass) that the compiler will
    /// produce the relevant not supported errors.
    #[test]
    fn unsupported_types_fail() {
        // Arrange
        let slice = "
            module Test;
            compact struct S
            {
                v: AnyClass,
            }
        ";
        let expected: [&dyn ErrorType; 3] = [
            &RuleKind::from(InvalidEncodingKind::UnsupportedType {
                type_string: "AnyClass".to_owned(),
                encoding: "2".to_owned(),
            }),
            &Note::new("file is using the Slice2 encoding by default"),
            &Note::new("to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'"),
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors_new!(error_reporter, expected);
    }

    /// Verifies that valid Slice2 types (bool, int8, uint8, int16, uint16, int32, uint32,
    /// varint32, varuint32, int64, uint64, varint62, varuint62, float32, float64, and string) will
    /// not produce any compiler errors.
    #[test_case("bool"; "bool")]
    #[test_case("int8"; "int8")]
    #[test_case("uint8"; "uint8")]
    #[test_case("int16"; "int16")]
    #[test_case("uint16"; "uint16")]
    #[test_case("int32"; "int32")]
    #[test_case("uint32"; "uint32")]
    #[test_case("varint32"; "varint32")]
    #[test_case("varuint32"; "varuint32")]
    #[test_case("int64"; "int64")]
    #[test_case("uint64"; "uint64")]
    #[test_case("varint62"; "varint62")]
    #[test_case("varuint62"; "varuint62")]
    #[test_case("float32"; "float32")]
    #[test_case("float64"; "float64")]
    #[test_case("string"; "string")]
    fn supported_types_succeed(value: &str) {
        // Arrange
        let slice = format!(
            "
            module Test;
            compact struct S
            {{
                v: {value},
            }}",
            value = value,
        );

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter);
    }

    #[test_case("uint8?"; "optional uint8")]
    #[test_case("uint16?"; "optional uint16")]
    #[test_case("uint32?"; "optional uint32")]
    #[test_case("uint64?"; "optional uint64")]
    #[test_case("int8?"; "optional int8")]
    #[test_case("int16?"; "optional int16")]
    #[test_case("int32?"; "optional int32")]
    #[test_case("int64?"; "optional int64")]
    #[test_case("varint32?"; "optional varint32")]
    #[test_case("varuint32?"; "optional varuint32")]
    #[test_case("varint62?"; "optional varint62")]
    #[test_case("varuint62?"; "optional varuint62")]
    #[test_case("string?"; "optional string")]
    #[test_case("bool?"; "optional bool")]
    #[test_case("sequence<int32>?"; "optional sequence")]
    #[test_case("float32?"; "optional float32")]
    #[test_case("float64?"; "optional float64")]
    fn supported_optional_types_succeed(value: &str) {
        // Arrange
        let slice = format!(
            "
                module Test;
                struct MyStruct {{
                    myVar: {value},
                }}
            ",
            value = value,
        );

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter);
    }
}
