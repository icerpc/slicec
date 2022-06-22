// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice1 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_errors;

    /// Verifies using the slice parser with Slice1 will emit errors when parsing
    /// non-compact structs.
    #[test]
    fn unsupported_fail() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            struct A {}
        ";
        let expected_errors = [
            "struct `A` is not supported by the Slice1 encoding",
            "file encoding was set to Slice1 here:",
            "structs must be `compact` to be supported by the Slice1 encoding",
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, expected_errors);
    }
}

mod slice2 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_errors;

    /// Verifies using the slice parser with Slice2 will emit errors when parsing
    /// structs that contain Slice1 types.
    #[test]
    fn slice1_types_fail() {
        // Arrange
        let slice = "
            module Test;
            struct A
            {
                c: AnyClass,
            }
        ";
        let expected_errors = [
            "the type `AnyClass` is not supported by the Slice2 encoding",
            "file is using the Slice2 encoding by default",
            "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'",
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, expected_errors);
    }

    /// Verifies using the slice parser with Slice2 will not emit errors when parsing
    /// structs that contain Slice2 types.
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
        assert_errors!(error_reporter);
    }
}

mod compact_structs {}
