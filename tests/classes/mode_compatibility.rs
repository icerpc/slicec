// Copyright (c) ZeroC, Inc.

mod slice2 {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use slicec::grammar::CompilationMode;

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
        let error = Error::NotSupportedInCompilationMode {
            kind: "class".to_owned(),
            identifier: "C".to_owned(),
            mode: CompilationMode::Slice2,
        };
        let expected = Diagnostic::new(error)
            .add_note("classes can only be defined in Slice1 mode", None)
            .add_note("this file's compilation mode is Slice2 by default", None);

        check_diagnostics(diagnostics, [expected]);
    }
}
