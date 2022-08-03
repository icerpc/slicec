// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice1 {

    use crate::assert_errors_new;
    use crate::helpers::parsing_helpers::parse_for_errors;
    use slice::errors::{ErrorKind, LogicKind};
    use slice::grammar::Encoding;

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

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        let expected = [
            LogicKind::NotSupportedWithEncoding("struct".to_owned(), "A".to_owned(), Encoding::Slice1).into(),
            ErrorKind::new_note("file encoding was set to Slice1 here:"),
            ErrorKind::new_note("structs must be `compact` to be supported by the Slice1 encoding"),
        ];
        assert_errors_new!(error_reporter, expected);
    }
}

mod slice2 {

    use crate::helpers::parsing_helpers::parse_for_errors;
    use crate::{assert_errors, assert_errors_new};
    use slice::errors::{ErrorKind, LogicKind};
    use slice::grammar::Encoding;

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

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        let expected: [ErrorKind; 3] = [
            LogicKind::UnsupportedType("AnyClass".to_owned(), Encoding::Slice2).into(),
            ErrorKind::new_note("file is using the Slice2 encoding by default"),
            ErrorKind::new_note(
                "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'",
            ),
        ];
        assert_errors_new!(error_reporter, expected);
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
