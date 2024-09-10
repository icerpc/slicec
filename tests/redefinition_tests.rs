// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod redefinition {
    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use slicec::slice_file::Span;

    #[test]
    fn redefinitions_of_the_same_type_are_disallowed() {
        // Arrange
        let slice = "
            module Test

            struct S {
                i: int32
            }

            struct S {
                i: int32
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::Redefinition {
            identifier: "S".to_owned(),
        })
        .set_span(&Span::new((8, 20).into(), (8, 21).into(), "string-0"))
        .add_note(
            "'S' was previously defined here",
            Some(&Span::new((4, 20).into(), (4, 21).into(), "string-0")),
        );

        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn redefinitions_of_different_types_are_disallowed() {
        // Arrange
        let slice = "
            module Test

            enum A { i }

            struct A {
                i: int32
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::Redefinition {
            identifier: "A".to_owned(),
        })
        .set_span(&Span::new((6, 20).into(), (6, 21).into(), "string-0"))
        .add_note(
            "'A' was previously defined here",
            Some(&Span::new((4, 18).into(), (4, 19).into(), "string-0")),
        );

        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn orthogonal_redefinitions_are_disallowed_separately() {
        // Arrange
        let slice = "
            module Test

            struct A {
                i: int32
                i: int64
            }

            interface A {
                i(i: int32)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let i_error = Diagnostic::new(Error::Redefinition {
            identifier: "i".to_owned(),
        })
        .set_span(&Span::new((6, 17).into(), (6, 18).into(), "string-0"))
        .add_note(
            "'i' was previously defined here",
            Some(&Span::new((5, 17).into(), (5, 18).into(), "string-0")),
        );

        let s_error = Diagnostic::new(Error::Redefinition {
            identifier: "A".to_owned(),
        })
        .set_span(&Span::new((9, 23).into(), (9, 24).into(), "string-0"))
        .add_note(
            "'A' was previously defined here",
            Some(&Span::new((4, 20).into(), (4, 21).into(), "string-0")),
        );

        check_diagnostics(diagnostics, [i_error, s_error]);
    }

    #[test]
    fn multiple_redefinitions_trigger_separate_errors() {
        // Arrange
        let slice = "
            module Test

            struct A {
                i: int8

                b: bool
                b: bool
            }

            interface A {
                i(b: bool, b: bool) -> (b: bool, b: bool)

                b()
                b()
            }

            enum A { i, b, b }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = [
            // The struct fields
            Diagnostic::new(Error::Redefinition {
                identifier: "b".to_owned(),
            })
            .set_span(&Span::new((8, 17).into(), (8, 18).into(), "string-0")),
            // The interface
            Diagnostic::new(Error::Redefinition {
                identifier: "A".to_owned(),
            })
            .set_span(&Span::new((11, 23).into(), (11, 24).into(), "string-0")),
            // The operation
            Diagnostic::new(Error::Redefinition {
                identifier: "b".to_owned(),
            })
            .set_span(&Span::new((15, 17).into(), (15, 18).into(), "string-0")),
            // The parameter
            Diagnostic::new(Error::Redefinition {
                identifier: "b".to_owned(),
            })
            .set_span(&Span::new((12, 28).into(), (12, 29).into(), "string-0")),
            // The return member
            Diagnostic::new(Error::Redefinition {
                identifier: "b".to_owned(),
            })
            .set_span(&Span::new((12, 50).into(), (12, 51).into(), "string-0")),
            // The enum
            Diagnostic::new(Error::Redefinition {
                identifier: "A".to_owned(),
            })
            .set_span(&Span::new((18, 18).into(), (18, 19).into(), "string-0")),
            // The enumerator
            Diagnostic::new(Error::Redefinition {
                identifier: "b".to_owned(),
            })
            .set_span(&Span::new((18, 28).into(), (18, 29).into(), "string-0")),
        ];

        check_diagnostics(diagnostics, expected);
    }

    #[test]
    fn cross_module_redefinitions_are_disallowed() {
        // Arrange
        let slice1 = "
            module Foo
            struct Bar {}
        ";
        let slice2 = "
            module Foo
            custom Bar
        ";

        // Act
        let diagnostics = parse_multiple_for_diagnostics(&[slice1, slice2]);

        // Assert
        let expected = Diagnostic::new(Error::Redefinition {
            identifier: "Bar".to_owned(),
        })
        .set_span(&Span::new((3, 20).into(), (3, 23).into(), "string-1"))
        .add_note(
            "'Bar' was previously defined here",
            Some(&Span::new((3, 20).into(), (3, 23).into(), "string-0")),
        );

        check_diagnostics(diagnostics, [expected]);
    }
}
