// Copyright (c) ZeroC, Inc.

mod slice1 {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};

    /// Verifies that exceptions cannot be used as data types while in Slice1 mode.
    #[test]
    fn can_not_be_fields() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test

            exception E {}

            compact struct S {
                e: E
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::ExceptionAsDataType);

        check_diagnostics(diagnostics, [expected]);
    }
}

mod slice2 {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use slicec::grammar::CompilationMode;

    /// Verifies that exception inheritance is disallowed while in Slice2 mode.
    #[test]
    fn inheritance_fails() {
        // Arrange
        let slice = "
            module Test

            exception A {}

            exception B : A {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::NotSupportedInCompilationMode {
            kind: "exception".to_owned(),
            identifier: "B".to_owned(),
            mode: CompilationMode::Slice2,
        })
        .add_note("exception inheritance can only be used in Slice1 mode", None)
        .add_note("this file's compilation mode is Slice2 by default", None);

        check_diagnostics(diagnostics, [expected]);
    }

    /// Verifies that exceptions can be used as data types while in Slice2 mode.
    #[test]
    fn can_be_fields() {
        // Arrange
        let slice = "
            module Test

            exception E {}

            struct S {
                e: E
            }
        ";

        // Act/Assert
        assert_parses(slice);
    }

    #[test]
    fn slice1_only_exceptions_cannot_be_thrown_from_slice2_operation() {
        // Arrange
        let slice1 = "
            mode = Slice1
            module Test

            exception E {
                a: AnyClass
            }
        ";

        let slice2 = "
            module Test

            interface I {
                op() throws E
            }
        ";

        // Act
        let diagnostics = parse_multiple_for_diagnostics(&[slice1, slice2]);

        // Assert
        let expected = Diagnostic::new(Error::UnsupportedType {
            kind: "E".to_owned(),
            mode: CompilationMode::Slice2,
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn cannot_throw_any_exception() {
        // Arrange
        let slice = "
            module Test

            interface I {
                op() throws AnyException
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::AnyExceptionNotSupported);
        check_diagnostics(diagnostics, [expected]);
    }
}
