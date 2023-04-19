// Copyright (c) ZeroC, Inc.

pub mod test_helpers;

use crate::test_helpers::*;
use slice::diagnostics::{Diagnostic, Error};

mod container {
    use super::*;
    use test_case::test_case;

    #[test]
    fn classes_can_contain_cycles() {
        // Arrange
        let slice = "
            encoding = Slice1
            module Test

            class C {
                c: C
            }
        ";

        // Act/Assert
        assert_parses(slice)
    }

    #[test_case("struct")]
    #[test_case("exception")]
    fn direct_cycles_are_disallowed(kind: &str) {
        // Arrange
        let slice = format!(
            "
            module Test

            {kind} Container {{
                c: Container
            }}
            "
        );

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::InfiniteSizeCycle {
            type_id: "Test::Container".to_owned(),
            cycle: "Test::Container -> Test::Container".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test_case("struct")]
    #[test_case("exception")]
    fn indirect_cycles_are_disallowed(kind: &str) {
        // Arrange
        let slice = format!(
            "
            module Test

            {kind} Container {{
                i: Inner
            }}

            struct Inner {{
                c: Container
            }}
            "
        );

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = [
            Diagnostic::new(Error::InfiniteSizeCycle {
                type_id: "Test::Container".to_owned(),
                cycle: "Test::Container -> Test::Inner -> Test::Container".to_owned(),
            }),
            Diagnostic::new(Error::InfiniteSizeCycle {
                type_id: "Test::Inner".to_owned(),
                cycle: "Test::Inner -> Test::Container -> Test::Inner".to_owned(),
            }),
        ];
        check_diagnostics(diagnostics, expected);
    }

    #[test_case("struct")]
    #[test_case("exception")]
    fn using_a_cyclic_type_is_not_flagged(kind: &str) {
        // Arrange
        let slice = format!(
            "
            module Test

            {kind} OnlyUsesACyclicType {{
                c: Container
            }}

            {kind} Container {{
                c: Container
            }}
            "
        );

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert: only `Container` should be marked as cyclic here.
        let expected = Diagnostic::new(Error::InfiniteSizeCycle {
            type_id: "Test::Container".to_owned(),
            cycle: "Test::Container -> Test::Container".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn only_one_error_is_emitted_per_cycle() {
        // Arrange
        let slice = format!(
            "
            module Test

            struct Container {{
                c1: Container
                c2: Container
                c3: Container
            }}
            "
        );

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert: only one error is emitting, despite multiple cyclic paths existing.
        let expected = Diagnostic::new(Error::InfiniteSizeCycle {
            type_id: "Test::Container".to_owned(),
            cycle: "Test::Container -> Test::Container".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }
}

#[rustfmt::skip]
mod type_aliases {
    use super::*;

    #[test]
    fn direct_cycles_are_disallowed() {
        // Arrange
        let slice = "
            module Test
            typealias Foo = Foo
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = [
            Diagnostic::new(Error::SelfReferentialTypeAliasNeedsConcreteType {
                identifier: "Test::Foo".to_owned(),
            })
            .add_note("failed to resolve type due to a cycle in its definition", None)
            .add_note("cycle: Test::Foo -> Test::Foo".to_owned(), None),

            Diagnostic::new(Error::DoesNotExist {
                identifier: "Test::Foo".to_owned(),
            }),
        ];

        check_diagnostics(diagnostics, expected);
    }

    #[test]
    fn indirect_cycles_are_disallowed() {
        // Arrange
        let slice = "
            module Test
    
            typealias Foo = Bar
            typealias Bar = Foo
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = [
            Diagnostic::new(Error::SelfReferentialTypeAliasNeedsConcreteType {
                identifier: "Test::Bar".to_owned(),
            })
            .add_note("failed to resolve type due to a cycle in its definition", None)
            .add_note("cycle: Test::Bar -> Test::Foo -> Test::Bar".to_owned(), None),

            Diagnostic::new(Error::DoesNotExist {
                identifier: "Test::Bar".to_owned(),
            }),

            Diagnostic::new(Error::SelfReferentialTypeAliasNeedsConcreteType {
                identifier: "Test::Foo".to_owned(),
            })
            .add_note("failed to resolve type due to a cycle in its definition", None)
            .add_note("cycle: Test::Foo -> Test::Bar -> Test::Foo".to_owned(), None),

            Diagnostic::new(Error::DoesNotExist {
                identifier: "Test::Foo".to_owned(),
            }),
        ];
        check_diagnostics(diagnostics, expected);
    }

    #[test]
    fn using_a_cyclic_type_is_not_flagged() {
        // Arrange
        let slice = "
            module Test

            typealias OnlyUsesACyclicType = Foo
            typealias Foo = Foo
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert: only `Foo` should be marked as cyclic here.
        let expected = [
            Diagnostic::new(Error::SelfReferentialTypeAliasNeedsConcreteType {
                identifier: "Test::Foo".to_owned(),
            })
            .add_note("failed to resolve type due to a cycle in its definition", None)
            .add_note("cycle: Test::Foo -> Test::Foo".to_owned(), None),

            Diagnostic::new(Error::DoesNotExist {
                identifier: "Test::Foo".to_owned(),
            }),

            Diagnostic::new(Error::SelfReferentialTypeAliasNeedsConcreteType {
                identifier: "Test::Foo".to_owned(),
            })
            .add_note("failed to resolve type due to a cycle in its definition", None)
            .add_note("cycle: Test::Foo -> Test::Foo".to_owned(), None),

            Diagnostic::new(Error::DoesNotExist {
                identifier: "Test::Foo".to_owned(),
            }),
        ];

        check_diagnostics(diagnostics, expected);
    }
}
