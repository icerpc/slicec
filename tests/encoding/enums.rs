// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice1 {

    use super::*;

    /// Verifies that the slice parser with the Slice 1 encoding emits errors when parsing an enum
    /// that has an underlying type.
    #[test]
    fn underlying_types_fail() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            enum E : int32 {}
            ";
        let expected_errors = &[
            "enums with underlying types are not supported by the Slice 1 encoding",
            "file encoding was set to the Slice 1 encoding here:",
        ];

        // Act
        let error_reporter = parse(slice);

        // Assert
        error_reporter.assert_errors(expected_errors);
    }
}
