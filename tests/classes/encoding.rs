// Copyright (c) ZeroC, Inc.

mod slice2 {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use slicec::grammar::Mode;

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
            mode: Mode::Slice2.to_string(),
        };
        let expected = Diagnostic::new(error)
            .add_note("classes are only supported by the Slice1 mode", None)
            .add_note("file is using Slice2 mode by default", None);

        check_diagnostics(diagnostics, [expected]);
    }
}
