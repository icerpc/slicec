// Copyright (c) ZeroC, Inc.

mod slice1 {

    use crate::helpers::parsing_helpers::*;
    use slice::diagnostics::{Error, ErrorKind};
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
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::NotSupportedWithEncoding {
            kind: "struct".to_owned(),
            identifier: "A".to_owned(),
            encoding: Encoding::Slice1,
        })
        .add_note("structs must be `compact` to be supported by the Slice1 encoding", None)
        .add_note("file encoding was set to Slice1 here:", None);

        check_diagnostics(diagnostics, [expected]);
    }
}

mod slice2 {

    use crate::helpers::parsing_helpers::*;
    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::Encoding;

    /// Verifies using the slice parser with Slice2 will emit errors when parsing
    /// structs that contain Slice1 types.
    #[test]
    fn slice1_types_fail() {
        // Arrange
        let slice = "
            module Test;

            struct A {
                c: AnyClass,
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::UnsupportedType {
            kind: "AnyClass".to_owned(),
            encoding: Encoding::Slice2,
        })
        .add_note("file is using the Slice2 encoding by default", None)
        .add_note(
            "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'",
            None,
        );

        check_diagnostics(diagnostics, [expected]);
    }

    /// Verifies using the slice parser with Slice2 will not emit errors when parsing
    /// structs that contain Slice2 types.
    #[test]
    fn slice2_types_succeed() {
        // Arrange
        let slice = "
            module Test;

            struct A {
                i: int32,
                s: string?,
            }
        ";

        // Act/Assert
        assert_parses(slice);
    }
}
