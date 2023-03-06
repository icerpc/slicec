// Copyright (c) ZeroC, Inc.

pub mod test_helpers;

mod tags {

    use crate::test_helpers::*;
    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::*;
    use test_case::test_case;

    #[test]
    fn tagged_data_members_must_be_optional() {
        // Arrange
        let slice = "
            encoding = 1
            module Test
            class C {
                i: int32
                s: string
                b: tag(10) bool
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::TaggedMemberMustBeOptional {
            member_identifier: "b".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn tagged_parameters_must_be_optional() {
        // Arrange
        let slice = "
            encoding = 1
            module Test
            interface I {
                op(myParam: tag(10) int32)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::TaggedMemberMustBeOptional {
            member_identifier: "myParam".to_string(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn non_tagged_optional_types_fail() {
        // Arrange
        let slice = "
            encoding = 1
            module Test
            interface I {
                myOp(a: int32?)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::OptionalsNotSupported {
            encoding: Encoding::Slice1,
        })
        .add_note("file encoding was set to Slice1 here:", None);

        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn tagged_parameters_must_be_after_required_parameters() {
        // Arrange
        let slice = "
            encoding = 1
            module Test
            interface I {
                op(p1: int32, p2: tag(10) int32?, p3: int32, p4: int32, p5: tag(20) int32?)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = [
            Error::new(ErrorKind::RequiredMustPrecedeOptional {
                parameter_identifier: "p3".to_owned(),
            }),
            Error::new(ErrorKind::RequiredMustPrecedeOptional {
                parameter_identifier: "p4".to_owned(),
            }),
        ];
        check_diagnostics(diagnostics, expected);
    }

    #[test]
    fn cannot_tag_a_class() {
        // Arrange
        let slice = "
            encoding = 1
            module Test

            class C {}

            interface I {
                op(c: tag(1) C?)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::CannotTagClass {
            member_identifier: "c".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn cannot_tag_a_container_that_contains_a_class() {
        // Arrange
        let slice = "
            encoding = 1
            module Test

            class C {}

            compact struct S {
                c: C
            }

            interface I {
                op(s: tag(1) S?)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::CannotTagContainingClass {
            member_identifier: "s".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn valid_tag() {
        // Arrange
        let slice = "
            module Test
            struct S {
                a: tag(1) int32?
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let data_member = ast.find_element::<DataMember>("Test::S::a").unwrap();

        assert_eq!(data_member.tag(), Some(1));
        assert!(data_member.data_type.is_optional);
    }

    #[test]
    fn cannot_have_duplicate_tags() {
        // Arrange
        let slice = "
            module Test
            struct S {
                a: tag(1) int32?
                b: tag(1) int32?
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::CannotHaveDuplicateTag {
            member_identifier: "b".to_owned(),
        })
        .add_note("The data member 'a' has previous used the tag value '1'", None);

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
                testOp(a: tag({value}) int32?)
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
                    testOp(a: tag({value}) int32?)
                }}
            "
        );

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::TagValueOutOfBounds);
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn cannot_have_tag_with_value_smaller_than_minimum() {
        // Arrange
        let slice = "
            module Test
            interface I {
                testOp(a: tag(-1) int32?)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::TagValueOutOfBounds);
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn strings_invalid_as_tag_value() {
        // Arrange
        let slice = "
            module Test
            interface I {
                testOp(a: tag(\"test string\") int32?)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::Syntax {
            message: "expected one of '-' or 'integer literal', but found 'test string'".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }
}
