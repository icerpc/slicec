// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod tags {

    use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_errors};
    use crate::{assert_errors, assert_errors_new};
    use slice::errors::*;
    use slice::grammar::*;
    use slice::parse_from_string;

    #[test]
    fn tagged_data_members_must_be_optional() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            class C {
                i: int32,
                s: string,
                b: tag(10) bool,
            }
        ";
        let expected = RuleKind::InvalidMember("b".to_owned(), InvalidMemberKind::MustBeOptional);
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors_new!(error_reporter, [&expected]);
    }

    #[test]
    fn tagged_parameters_must_be_optional() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            interface I {
                op(myParam: tag(10) int32);
            }
        ";
        let expected = RuleKind::InvalidMember("myParam".to_owned(), InvalidMemberKind::MustBeOptional);

        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors_new!(error_reporter, [&expected]);
    }

    #[test]
    #[ignore = "reason: TODO Need to update AST Error emission"]
    fn non_tagged_optional_types_fail() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            interface I {
                myOp(a: int32?);
            }
        ";

        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, [
            "optional types are not supported by the Slice1 encoding (except for classes, proxies, and with tags)",
            "file encoding was set to Slice1 here:",
        ]);
    }

    #[test]
    fn tagged_parameters_must_be_after_required_parameters() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            interface I {
                op(p1: int32, p2: tag(10) int32?, p3: int32, p4: int32, p5: tag(20) int32?);
            }
        ";
        let expected: [&dyn ErrorType; 2] = [
            &RuleKind::InvalidParameter("p3".to_owned(), InvalidParameterKind::RequiredParametersMustBeFirst),
            &RuleKind::InvalidParameter("p4".to_owned(), InvalidParameterKind::RequiredParametersMustBeFirst),
        ];
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors_new!(error_reporter, expected);
    }

    #[test]
    fn cannot_tag_a_class() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;

            class C {}

            interface I {
                op(c: tag(1) C?);
            }
        ";
        let expected = RuleKind::InvalidMember("c".to_owned(), InvalidMemberKind::CannotBeClass);

        // Act
        let errors = parse_for_errors(slice);

        // Assert
        assert_errors_new!(errors, [&expected]);
    }

    #[test]
    fn cannot_tag_a_container_that_contains_a_class() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;

            class C {}
            compact struct S {
                c: C,
            }

            interface I {
                op(s: tag(1) S?);
            }
        ";
        let expected = RuleKind::InvalidMember("s".to_owned(), InvalidMemberKind::CannotContainClasses);

        // Act
        let errors = parse_for_errors(slice);

        // Assert
        assert_errors_new!(errors, [&expected]);
    }

    #[test]
    fn valid_tag() {
        // Arrange
        let slice = "
            module Test;
            struct S {
                a: tag(1) int32?,
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
            module Test;
            struct S {
                a: tag(1) int32?,
                b: tag(1) int32?,
            }
        ";

        // Act
        let error_reporter = parse_for_errors(slice);

        let expected: [&dyn ErrorType; 2] = [
            &RuleKind::InvalidTag("b".to_owned(), InvalidTagKind::DuplicateTag),
            &Note::new("The data member `a` has previous used the tag value `1`"),
        ];

        // Assert
        assert_errors_new!(error_reporter, expected);
    }

    #[test]
    #[ignore = "reason: TODO Need to update AST Error emission"]
    fn cannot_have_tag_with_value_larger_than_max() {
        // Arrange
        let max_value = i32::MAX as i64;
        let slice = format!(
            "
                module Test;
                interface I {{
                    testOp(a: tag({value}) int32?);
                }}
            ",
            value = max_value + 1
        );
        let expected = RuleKind::InvalidTag("a".to_owned(), InvalidTagKind::MustBeBounded);

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors_new!(error_reporter, [&expected]);
    }

    #[test]
    #[ignore = "reason: TODO Need to update AST Error emission"]
    fn cannot_have_tag_with_value_smaller_than_minimum() {
        // Arrange
        let slice = format!(
            "
                module Test;
                interface I {{
                    testOp(a: tag({value}) int32?);
                }}
            ",
            value = -1
        );
        let expected = RuleKind::InvalidTag("a".to_owned(), InvalidTagKind::MustBePositive);

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors_new!(error_reporter, [&expected]);
    }

    #[test] // TODO: We should not be panicking here. We should be returning an error.
    fn strings_invalid_as_tag_value() {
        // Arrange
        let slice = "
            module Test;
            interface I {
                testOp(a: tag(\"test string\") int32?);
            }
        ";

        // Act
        let err = parse_from_string(slice).err();

        // Assert
        assert!(err.is_some());
    }
}
