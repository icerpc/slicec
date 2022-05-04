// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::parse_for_errors;

mod slice1 {

    use super::*;

    /// Verifies that the slice parser with the Slice 1 encoding emits errors when parsing an
    /// exception that is a data member.
    #[test]
    fn can_not_be_data_members() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            exception E {}
            compact struct S
            {
                e: E,
            }
            ";
        let expected_errors = &[
            "type 'Test::S' isn't supported by its file's Slice encoding",
            "file encoding was set to the Slice 1 encoding here:",
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        error_reporter.assert_errors(expected_errors);
    }
}

mod slice2 {

    use super::*;

    /// Verifies that the slice parser with the Slice 2 encoding emits errors when parsing an
    /// exception that inherits from another exception.
    #[test]
    fn inheritance_fails() {
        // Arrange
        let slice = "
            module Test;
            exception A {}
            exception B : A {}
            ";
        let expected_errors = &[
            "exception inheritance is only supported by the Slice 1 encoding",
            "file encoding was set to the Slice 2 encoding here:",
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        error_reporter.assert_errors(expected_errors);
    }

    /// Verifies that the slice parser with the Slice 2 encoding does not emit errors when parsing
    /// exceptions that are data members.
    #[test]
    fn can_be_data_members() {
        // Arrange
        let slice = "
            module Test;
            exception E {}
            struct S
            {
                e: E,
            }
            ";

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert!(!error_reporter.has_errors(true));
    }
}
