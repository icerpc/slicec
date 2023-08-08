// Copyright (c) ZeroC, Inc.

mod slice1 {
    use crate::test_helpers::*;

    #[test]
    fn can_define_exceptions() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test

            exception E {}
        ";

        // Act / Assert
        assert_parses(slice);
    }
}

mod slice2 {
    use crate::test_helpers::*;
    use slicec::grammar::CompilationMode;
    use slicec::diagnostics::{Diagnostic, Error};

    #[test]
    fn cannot_define_exceptions() {
        // Arrange
        let slice = "
            mode = Slice2
            module Test

            exception E {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::NotSupportedInCompilationMode {
            kind: "exception".to_owned(),
            identifier: "E".to_owned(),
            mode: CompilationMode::Slice2,
        });
        check_diagnostics(diagnostics, [expected]);
    }
}
