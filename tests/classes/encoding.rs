// Copyright (c) ZeroC, Inc.

mod slice2 {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use slicec::grammar::Encoding;

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
        let error = Error::NotSupportedWithMode {
            kind: "class".to_owned(),
            identifier: "C".to_owned(),
            mode: Encoding::Slice2.to_string(),
        };
        let expected = Diagnostic::new(error)
            .add_note("classes are only supported by the Slice1 encoding", None)
            .add_note("file is using the Slice2 encoding by default", None);

        check_diagnostics(diagnostics, [expected]);
    }
}
