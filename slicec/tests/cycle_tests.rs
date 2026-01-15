// Copyright (c) ZeroC, Inc.

mod test_helpers;

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};

mod container {
    use super::*;

    #[test]
    fn classes_can_contain_cycles() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test

            class C {
                c: C
            }
        ";

        // Act/Assert
        assert_parses(slice)
    }

    #[test]
    fn direct_cycles_are_disallowed() {
        // Arrange
        let slice = "
            module Test

            struct Container {
                c: Container
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::InfiniteSizeCycle {
            type_id: "Test::Container".to_owned(),
            cycle: "Test::Container -> Test::Container".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn indirect_cycles_are_disallowed() {
        // Arrange
        let slice = "
            module Test

            struct Container {
                i: Inner
            }

            struct Inner {
                c: Container
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::InfiniteSizeCycle {
            type_id: "Test::Container".to_owned(),
            cycle: "Test::Container -> Test::Inner -> Test::Container".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn using_a_cyclic_type_is_not_flagged() {
        // Arrange
        let slice = "
            module Test

            struct OnlyUsesACyclicType {
                c: Container
            }

            struct Container {
                c: Container
            }
        ";

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
        let slice = "
            module Test

            struct Container {
                c1: Container
                c2: Container
                c3: Container
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert: only one error is emitted, despite multiple cyclic paths existing.
        let expected = Diagnostic::new(Error::InfiniteSizeCycle {
            type_id: "Test::Container".to_owned(),
            cycle: "Test::Container -> Test::Container".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn duplicate_cycles_are_not_reported_multiple_times() {
        // Arrange
        let slice = "
            module Test

            struct A { b: B, c: C }
            struct B { a: A, c: C }
            struct C { a: A, b: B }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert: There are technically 18 cycles in the above Slice, but only 4 are unique cycles.
        let expected = [
            Diagnostic::new(Error::InfiniteSizeCycle {
                type_id: "Test::A".to_owned(),
                cycle: "Test::A -> Test::B -> Test::A".to_owned(),
            }),
            Diagnostic::new(Error::InfiniteSizeCycle {
                type_id: "Test::A".to_owned(),
                cycle: "Test::A -> Test::B -> Test::C -> Test::A".to_owned(),
            }),
            Diagnostic::new(Error::InfiniteSizeCycle {
                type_id: "Test::A".to_owned(),
                cycle: "Test::A -> Test::C -> Test::A".to_owned(),
            }),
            Diagnostic::new(Error::InfiniteSizeCycle {
                type_id: "Test::B".to_owned(),
                cycle: "Test::B -> Test::C -> Test::B".to_owned(),
            }),
        ];
        check_diagnostics(diagnostics, expected);
    }
}

mod builtin {
    use super::*;
    use slicec::slice_file::Span;

    #[test]
    fn cycles_through_results_are_disallowed() {
        // Arrange
        let slice = "
            module Test

            struct Foo {
                f: Result<Foo, bool>
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::InfiniteSizeCycle {
            type_id: "Test::Foo".to_owned(),
            cycle: "Test::Foo -> Test::Foo".to_owned(),
        })
        .add_note(
            "struct 'Foo' contains a field named 'f' that is of type 'Result<Foo, bool>'",
            Some(&Span::new((5, 17).into(), (5, 37).into(), "string-0")),
        );

        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn cycles_through_sequences_are_disallowed() {
        // Arrange
        let slice = "
            module Test

            struct Foo {
                f: Sequence<Foo>
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::InfiniteSizeCycle {
            type_id: "Test::Foo".to_owned(),
            cycle: "Test::Foo -> Test::Foo".to_owned(),
        })
        .add_note(
            "struct 'Foo' contains a field named 'f' that is of type 'Sequence<Foo>'",
            Some(&Span::new((5, 17).into(), (5, 33).into(), "string-0")),
        );

        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn cycles_through_dictionaries_are_disallowed() {
        // Arrange
        let slice = "
            module Test

            struct Foo {
                f: Dictionary<Foo, bool>
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::InfiniteSizeCycle {
            type_id: "Test::Foo".to_owned(),
            cycle: "Test::Foo -> Test::Foo".to_owned(),
        })
        .add_note(
            "struct 'Foo' contains a field named 'f' that is of type 'Dictionary<Foo, bool>'",
            Some(&Span::new((5, 17).into(), (5, 41).into(), "string-0")),
        );

        check_diagnostics(diagnostics, [expected]);
    }
}

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
