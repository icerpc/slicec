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
    #[ignore] // Encoding 1 with compact struct containing exceptions is not supported, compilation should
              // fail
    fn can_not_be_data_members() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            exception E {}
            compact struct S
            {
                e: E,
            }";
        let expected_errors = &[
            "exception inheritance is only supported by the Slice 1 encoding",
            "file encoding was set to the Slice 2 encoding here:",
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
    fn no_inheritance() {
        // Arrange
        let slice = "
            encoding = 2;
            module Test;
            exception A {}
            exception B : A {}
            ";
        let expected_errors = &[
            "exception inheritance is only supported by the Slice 1 encoding",
            "file encoding was set to the Slice 2 encoding here:",
        ];

        // Act
        let error_reporter = parse(slice);

        // Assert
        error_reporter.assert_errors(expected_errors);
    }

    #[test]
    fn can_be_data_members() {
        // Arrange
        let slice = "
            encoding = 2;
            module Test;
            exception E {}
            struct S
            {
                e: E,
            }";

        // Act
        let error_reporter = parse(slice);

        // Assert
        assert!(!error_reporter.has_errors(true));
    }
}
