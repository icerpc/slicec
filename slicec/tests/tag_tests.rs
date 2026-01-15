// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod tags {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use slicec::grammar::*;
    use test_case::test_case;

    #[test]
    fn tagged_fields_must_be_optional() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test
            class C {
                i: int32
                s: string
                tag(10) b: bool
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::TaggedMemberMustBeOptional {
            identifier: "b".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn tagged_parameters_must_be_optional() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test
            interface I {
                op(tag(10) myParam: int32)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::TaggedMemberMustBeOptional {
            identifier: "myParam".to_string(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn tagged_parameters_can_be_in_any_order() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test
            interface I {
                op(p1: int32, tag(10) p2: int32?, p3: int32, p4: int32, tag(20) p5: int32?)
            }
        ";

        // Act/Assert
        assert_parses(slice);
    }

    #[test]
    fn cannot_tag_a_class() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test

            class C {}

            interface I {
                op(tag(1) c: C?)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::CannotTagClass {
            identifier: "c".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn cannot_tag_a_container_that_contains_a_class() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test

            class C {}

            compact struct S {
                c: C
            }

            interface I {
                op(tag(1) s: S?)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::CannotTagContainingClass {
            identifier: "s".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn valid_tag() {
        // Arrange
        let slice = "
            module Test
            struct S {
                tag(1) a: int32?
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();

        assert_eq!(field.tag(), Some(1));
        assert!(field.data_type.is_optional);
    }

    #[test]
    fn cannot_have_duplicate_tags() {
        // Arrange
        let slice = "
            module Test
            struct S {
                tag(1) a: int32?
                tag(1) b: int32?
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::CannotHaveDuplicateTag {
            identifier: "b".to_owned(),
        })
        .add_note("The tag '1' is already being used by member 'a'", None);

        check_diagnostics(diagnostics, [expected]);
    }

    #[test_case(0)]
    #[test_case(i32::MAX / 2)]
    #[test_case(i32::MAX)]
    fn valid_tag_value(value: i32) {
        // Arrange
        let slice = format!(
            "
            module Test
            interface I {{
                testOp(tag({value}) a: int32?)
            }}
            "
        );

        // Act/Assert
        assert_parses(slice);
    }

    #[test_case(77757348128678234_i64 ; "Random large value")]
    #[test_case((i32::MAX as i64) + 1; "Slightly over")]
    fn cannot_have_tag_with_value_larger_than_max(value: i64) {
        // Arrange
        let slice = format!(
            "
                module Test
                interface I {{
                    testOp(tag({value}) a: int32?)
                }}
            "
        );

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::TagValueOutOfBounds);
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn cannot_have_tag_with_value_smaller_than_minimum() {
        // Arrange
        let slice = "
            module Test
            interface I {
                testOp(tag(-1) a: int32?)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::TagValueOutOfBounds);
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn strings_invalid_as_tag_value() {
        // Arrange
        let slice = "
            module Test
            interface I {
                testOp(tag(\"test string\") a: int32?)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::Syntax {
            message: "expected one of 'integer literal' or '-', but found 'test string'".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }
}
