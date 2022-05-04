// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::parse_for_errors;

mod slice1 {

    use super::*;

    /// Verifies using the slice parser with the Slice 1 encoding will emit errors when parsing
    /// non-compact structs.
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
        let error_reporter = parse_for_errors(slice);

        // Assert
        error_reporter.assert_errors(expected_errors);
    }
}

mod slice2 {

    use super::*;

    /// Verifies using the slice parser with the Slice 2 encoding will emit errors when parsing
    /// structs that contain Slice 1 types.
    #[test]
    fn slice1_types_fail() {
        // Arrange
        let slice = "
        module Test;
        struct A
        {
            c: AnyClass
        }
        ";
        let expected_errors = &[
            "'AnyClass' is not supported by the Slice 2 encoding",
            "file is using the Slice 2 encoding by default",
            "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'",
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        error_reporter.assert_errors(expected_errors);
    }

    /// Verifies using the slice parser with the Slice 2 encoding will not emit errors when parsing
    /// structs that contain Slice 2 types.
    #[test]
    fn slice2_types_succeed() {
        // Arrange
        let slice = "
            module Test;
            trait T;
            struct A
            {
                i: int32,
                s: string?,
                t: T,
            }
            ";

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert!(!error_reporter.has_errors(true));
    }
}

mod compact_structs {}
