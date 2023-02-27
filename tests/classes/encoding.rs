// Copyright (c) ZeroC, Inc.

mod slice2 {

    use crate::helpers::parsing_helpers::*;
    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::Encoding;

    #[test]
    fn unsupported_error() {
        // Arrange
        let slice = "
            module Test
            class C {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let error_kind = ErrorKind::NotSupportedWithEncoding {
            kind: "class".to_owned(),
            identifier: "C".to_owned(),
            encoding: Encoding::Slice2,
        };
        let expected = Error::new(error_kind)
            .add_note("classes are only supported by the Slice1 encoding", None)
            .add_note("file is using the Slice2 encoding by default", None)
            .add_note(
                "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1'",
                None,
            );

        check_diagnostics(diagnostics, [expected]);
    }
}
