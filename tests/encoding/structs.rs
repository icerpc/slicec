// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::error::ErrorReporter;
use slice::parse_from_string;

pub fn parse(slice: &str) -> ErrorReporter {
    let (_, error_reporter) = parse_from_string(slice).ok().unwrap();

    error_reporter
}

mod slice1 {

    use super::*;

    #[test]
    fn unsupported_fail() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            struct A {}
            ";
        let expected_errors = &[
            "non-compact structs are not supported by the Slice 1 encoding",
            "file encoding was set to the Slice 1 encoding here:",
        ];

        // Act
        let error_reporter = parse(slice);

        // Assert
        error_reporter.assert_errors(expected_errors);
    }
}

mod slice2 {

    use super::*;

    #[test]
    fn slice1_types_fail() {
        // Arrange
        let slice = "
        encoding = 2;
        module Test;
        struct A
        {
            c: AnyClass
        }";
        let expected_errors = &[
            "'AnyClass' is not supported by the Slice 2 encoding",
            "file encoding was set to the Slice 2 encoding here:",
        ];

        // Act
        let error_reporter = parse(slice);

        // Assert
        error_reporter.assert_errors(expected_errors);
    }

    #[test]
    fn slice2_types_succeed() {
        // Arrange
        let slice = "
            encoding = 2;
            module Test;
            trait T;
            struct A
            {
                i: int32,
                s: string?,
                t: T,
            }";

        // Act
        let error_reporter = parse(slice);

        // Assert
        assert!(!error_reporter.has_errors(true));
    }
}

mod compact_structs {}
