// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice1 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_diagnostics;
    use slice::diagnostics::{Error, ErrorKind, Note};
    use slice::grammar::Encoding;

    /// Verifies using the slice parser with Slice1 will emit errors when parsing
    /// non-compact structs.
    #[test]
    fn unsupported_fail() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;

            struct A
            {
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new_with_notes(
            ErrorKind::NotSupportedWithEncoding("struct".to_owned(), "A".to_owned(), Encoding::Slice1),
            None,
            vec![
                Note::new("file encoding was set to Slice1 here:", None),
                Note::new("structs must be `compact` to be supported by the Slice1 encoding", None),
            ],
        );
        assert_errors!(diagnostic_reporter, [&expected]);
    }
}

mod slice2 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_diagnostics;
    use slice::diagnostics::{Error, ErrorKind, Note};
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
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new_with_notes(
            ErrorKind::UnsupportedType("AnyClass".to_owned(), Encoding::Slice2),
            None,
            vec![
                Note::new("file is using the Slice2 encoding by default", None),
                Note::new(
                    "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'",
                    None,
                ),
            ],
        );

        assert_errors!(diagnostic_reporter, [&expected]);
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
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(diagnostic_reporter);
    }
}
