// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::parse_for_errors;

mod slice1 {

    use super::*;
    use test_case::test_case;

    /// Verifies that if Slice 1 is used with unsupported types (int8, uint16, uint32, varint32,
    /// varuint32, uint64, varint62, and varuint62) that the compiler will produce the relevant not
    /// supported errors.
    #[test_case("int8")]
    #[test_case("uint16")]
    #[test_case("uint32")]
    #[test_case("varint32")]
    #[test_case("varuint32")]
    #[test_case("uint64")]
    #[test_case("varint62")]
    #[test_case("varuint62")]
    fn unsupported_types_fail(value: &str) {
        // Test setup
        let slice = &format!(
            "
            encoding = 1;
            module Test;
            compact struct S
            {{
                v: {value},
            }}",
            value = value,
        );

        let expected_errors: &[&str] = &[
            &format!("'{}' is not supported by the Slice 1 encoding", value),
            "file encoding was set to the Slice 1 encoding here:",
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        error_reporter.assert_errors(expected_errors);
    }

    /// Verifies that valid Slice 1 types (bool, uint8, int16, int32, int64, float32, float64,
    /// string, and  AnyClass) will not produce any compiler errors.
    #[test_case("bool")]
    #[test_case("uint8")]
    #[test_case("int16")]
    #[test_case("int32")]
    #[test_case("int64")]
    #[test_case("float32")]
    #[test_case("float64")]
    #[test_case("string")]
    #[test_case("AnyClass")]
    fn supported_types_succeed(value: &str) {
        // Arrange
        let slice = &format!(
            "
            encoding = 1;
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
        assert!(!error_reporter.has_errors(true));
    }
}

mod slice2 {

    use super::*;
    use test_case::test_case;

    /// Verifies that if Slice 2 is used with unsupported types (AnyClass) that the compiler will
    /// produce the relevant not supported errors.
    #[test]
    fn unsupported_types_fail() {
        // Arrange
        let slice = "
            encoding = 2;
            module Test;
            compact struct S
            {{
                v: AnyClass,
            }}";
        let expected_errors: &[&str] = &[
            "'AnyClass' is not supported by the Slice 2 encoding",
            "file encoding was set to the Slice 2 encoding here:",
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        error_reporter.assert_errors(expected_errors);
    }

    /// Verifies that valid Slice 2 types (bool, int8, uint8, int16, uint16, int32, uint32,
    /// varint32, varuint32, int64, uint64, varint62, varuint62, float32, float64, and string) will
    /// not produce any compiler errors.
    #[test_case("bool")]
    #[test_case("int8")]
    #[test_case("uint8")]
    #[test_case("int16")]
    #[test_case("uint16")]
    #[test_case("int32")]
    #[test_case("uint32")]
    #[test_case("varint32")]
    #[test_case("varuint32")]
    #[test_case("int64")]
    #[test_case("uint64")]
    #[test_case("varint62")]
    #[test_case("varuint62")]
    #[test_case("float32")]
    #[test_case("float64")]
    #[test_case("string")]
    fn supported_types_succeed(value: &str) {
        // Arrange
        let slice = &format!(
            "
            encoding = 2;
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
        assert!(!error_reporter.has_errors(true));
    }

    #[test_case("uint8?")]
    #[test_case("uint16?")]
    #[test_case("uint32?")]
    #[test_case("uint64?")]
    #[test_case("int8?")]
    #[test_case("int16?")]
    #[test_case("int32?")]
    #[test_case("int64?")]
    #[test_case("varint32?")]
    #[test_case("varuint32?")]
    #[test_case("varint62?")]
    #[test_case("varuint62?")]
    #[test_case("string?")]
    #[test_case("bool?")]
    #[test_case("sequence<int32>?")]
    #[test_case("float32?")]
    #[test_case("float64?")]
    fn supported_optional_types_succeed(value: &str) {
        // Arrange
        let slice = &format!(
            "
            encoding = 2;
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
        assert!(!error_reporter.has_errors(true));
    }
}
