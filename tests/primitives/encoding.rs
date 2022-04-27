// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::parse_for_errors;

mod slice1 {

    use super::*;
    const ENCODING: &str = "1";

    /// Verifies that if Slice 1 is used with unsupported types (int8, uint16, uint32, varint32,
    /// varuint32, uint64, varint62, and varuint62) that the compiler will produce the relevant not
    /// supported errors.
    #[test]
    fn unsupported_types_fail() {
        // Test setup
        let type_cases = vec![
            "int8",
            "uint16",
            "uint32",
            "varint32",
            "varuint32",
            "uint64",
            "varint62",
            "varuint62",
        ];

        for value in type_cases.iter() {
            let errors: &[&str] = &[
                &format!("'{}' is not supported by the Slice 1 encoding", value),
                "file encoding was set to the Slice 1 encoding here:",
            ];
            test(value, errors)
        }

        fn test(value: &str, expected: &[&str]) {
            // Arrange
            let slice = &format!(
                "
                encoding = {encoding};
                module Test;
                compact struct S
                {{
                    v: {value},
                }}",
                encoding = ENCODING,
                value = value,
            );

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            error_reporter.assert_errors(expected);
        }
    }

    /// Verifies that valid Slice 1 types (bool, uint8, int16, int32, int64, float32, float64,
    /// string, and  AnyClass) will not produce any compiler errors.
    #[test]
    fn supported_types_succeed() {
        // Test setup
        let type_cases = vec![
            "bool", "uint8", "int16", "int32", "int64", "float32", "float64", "string", "AnyClass",
        ];

        for value in type_cases.iter() {
            test(value)
        }

        fn test(value: &str) {
            // Arrange
            let slice = &format!(
                "
                encoding = {encoding};
                module Test;
                compact struct S
                {{
                    v: {value},
                }}",
                encoding = ENCODING,
                value = value,
            );

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            assert!(!error_reporter.has_errors(true));
        }
    }
}

mod slice2 {

    use super::*;
    const ENCODING: &str = "2";

    /// Verifies that if Slice 2 is used with unsupported types (AnyClass) that the compiler will
    /// produce the relevant not supported errors.
    #[test]
    fn unsupported_types_fail() {
        // Test setup
        let type_cases = vec!["AnyClass"];
        for value in type_cases.iter() {
            let errors: &[&str] = &[
                &format!("'{}' is not supported by the Slice 2 encoding", value),
                "file encoding was set to the Slice 2 encoding here:",
            ];
            test(value, errors)
        }

        fn test(value: &str, expected: &[&str]) {
            // Arrange
            let slice = &format!(
                "
                encoding = {encoding};
                module Test;
                compact struct S
                {{
                    v: {value},
                }}",
                encoding = ENCODING,
                value = value,
            );

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            error_reporter.assert_errors(expected);
        }
    }

    /// Verifies that valid Slice 2 types (bool, int8, uint8, int16, uint16, int32, uint32,
    /// varint32, varuint32, int64, uint64, varint62, varuint62, float32, float64, and string) will
    /// not produce any compiler errors.
    #[test]
    fn supported_types_succeed() {
        // Test setup
        let type_cases = vec![
            "bool",
            "int8",
            "uint8",
            "int16",
            "uint16",
            "int32",
            "uint32",
            "varint32",
            "varuint32",
            "int64",
            "uint64",
            "varint62",
            "varuint62",
            "float32",
            "float64",
            "string",
        ];
        for value in type_cases.iter() {
            test(value)
        }

        fn test(value: &str) {
            // Arrange
            let slice = &format!(
                "
                encoding = {encoding};
                module Test;
                compact struct S
                {{
                    v: {value},
                }}",
                encoding = ENCODING,
                value = value,
            );

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            assert!(!error_reporter.has_errors(true));
        }
    }
}
