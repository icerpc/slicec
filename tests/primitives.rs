// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::parse_from_string;
use slice::error::ErrorReporter;

pub fn parse(slice: &str) -> ErrorReporter {
    let (_, error_reporter) = parse_from_string(slice).ok().unwrap();

    error_reporter
}


mod primitives {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn support_types_do_not_produce_errors() {
        // Test setup
        let type_cases = HashMap::from([
            ("2", vec!["bool", "int8", "uint8", "int16", "uint16", "int32", "uint32", "varint32", "varuint32",
                    "int64", "uint64", "varint62", "varuint62", "float32", "float64", "string"]),
            ("1", vec!["bool", "uint8", "int16", "int32", "int64", "float32","float64", "string", "AnyClass"])
            ]);
        for (encoding, value) in type_cases.iter().flat_map(|(encoding, value)| value.iter().map(move |v| (encoding, v))) {
            supported_types(encoding, value)
        }

        fn supported_types(encoding: &str, value: &str) {
            // Arrange
            let slice = &format!("
                encoding = {encoding};
                module Test;
                compact struct S
                {{
                    v: {value},
                }}",
            encoding = encoding,
            value = value,
            );

            // Act
            let error_reporter = parse(slice);

            // Assert
            assert!(!error_reporter.has_errors(true));
        }
    }

    #[test]
    fn unsupported_types_produce_error() {
        // Test setup
        let type_cases = HashMap::from([
            ("1", vec!["int8", "uint16","uint32", "varint32",  "varuint32", "uint64", "varint62", "varuint62"]),
            ("2", vec!["AnyClass"])
            ]);
        for (encoding, value) in type_cases.iter().flat_map(|(encoding, value)| value.iter().map(move |v| (encoding, v))) {
            let errors: &[&str] = &[
                &format!("'{}' is not supported by the Slice {} encoding", value, encoding),
                &format!("file encoding was set to the Slice {} encoding here:", encoding),
            ];
            unsupported_types(encoding, value, errors)
        }

        fn unsupported_types(encoding: &str, value: &str, expected: &[&str]) {
            // Arrange
            let slice = &format!("
                encoding = {encoding};
                module Test;
                compact struct S
                {{
                    v: {value},
                }}",
            encoding = encoding,
            value = value,
            );

            // Act
            let error_reporter = parse(slice);

            // Assert
            error_reporter.assert_errors(expected);
        }
    }
}