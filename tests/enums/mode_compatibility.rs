// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};
use slicec::grammar::CompilationMode;

#[test]
fn slice1_enums_can_be_used_by_slice2_definitions() {
    // Arrange
    let slice1 = "
        mode = Slice1
        module Test

        enum E {
            A,
            B = 4,
        }
    ";
    let slice2 = "
        module Test

        struct S { e: E}
    ";

    // Act/Assert: ensure it compiles without emitting any mode compatibility errors.
    _ = parse_multiple_for_ast(&[slice1, slice2]);
}

mod slice1 {
    use super::*;

    /// Verifies that underlying types are disallowed in Slice1 mode.
    #[test]
    fn underlying_types_fail() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test

            unchecked enum E : int32 {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::NotSupportedInCompilationMode {
            kind: "enum".to_owned(),
            identifier: "E".to_owned(),
            mode: CompilationMode::Slice1,
        })
        .add_note("enums defined in Slice1 mode cannot have underlying types", None);

        check_diagnostics(diagnostics, [expected]);
    }

    /// Verifies that enumerators with fields are disallowed in Slice1 mode.
    #[test]
    fn associated_fields_fail() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test

            unchecked enum E {
                A(b: bool)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::NotSupportedInCompilationMode {
            kind: "enum".to_owned(),
            identifier: "E".to_owned(),
            mode: CompilationMode::Slice1,
        })
        .add_note(
            "field syntax cannot be used with enumerators declared in Slice1 mode",
            None,
        );
        check_diagnostics(diagnostics, [expected]);
    }

    /// Verifies that even empty field lists are disallowed in Slice1 mode.
    #[test]
    fn empty_associated_fields_fail() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test

            unchecked enum E {
                A()
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::NotSupportedInCompilationMode {
            kind: "enum".to_owned(),
            identifier: "E".to_owned(),
            mode: CompilationMode::Slice1,
        })
        .add_note(
            "field syntax cannot be used with enumerators declared in Slice1 mode",
            None,
        );
        check_diagnostics(diagnostics, [expected]);
    }

    /// Verifies that compact enums are disallowed in Slice1 mode.
    #[test]
    fn compact_enums_fail() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test

            compact enum E {
                A
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::NotSupportedInCompilationMode {
            kind: "enum".to_owned(),
            identifier: "E".to_owned(),
            mode: CompilationMode::Slice1,
        })
        .add_note(
            "enums defined in Slice1 mode cannot be 'compact'",
            None,
        );
        check_diagnostics(diagnostics, [expected]);
    }
}
