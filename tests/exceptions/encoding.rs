// Copyright (c) ZeroC, Inc.

mod slice1 {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};

    /// Verifies that the slice parser with the Slice1 encoding emits errors when parsing an
    /// exception that is a field.
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
    use slicec::grammar::Mode;

    /// Verifies that the slice parser with the Slice2 encoding emits errors when parsing an
    /// exception that inherits from another exception.
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
        let expected = Diagnostic::new(Error::NotSupportedWithMode {
            kind: "exception".to_owned(),
            identifier: "B".to_owned(),
            mode: Mode::Slice2.to_string(),
        })
        .add_note("exception inheritance is only supported by the Slice1 mode", None)
        .add_note("file is using Slice2 mode by default", None);

        check_diagnostics(diagnostics, [expected]);
    }

    /// Verifies that the slice parser with the Slice2 encoding does not emit errors when parsing
    /// exceptions that are fields.
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

    /// Verify that exceptions which are only Slice1 encodable a Slice2 operation.
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
            mode: Mode::Slice2.to_string(),
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
