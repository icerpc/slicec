// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod tags {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_errors};
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

        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, [
            "invalid member `b`: tagged members must be optional",
        ]);
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

        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, [
            "invalid member `myParam`: tagged members must be optional",
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

        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, [
            "invalid parameter `p3`: required parameters must precede tagged parameters",
            "invalid parameter `p4`: required parameters must precede tagged parameters",
        ]);
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
        let expected_errors = ["invalid member `c`: tagged members cannot be classes"];

        // Act
        let errors = parse_for_errors(slice);

        // Assert
        assert_errors!(errors, expected_errors);
    }

    #[test]
    fn cannot_tag_a_container_that_contains_a_class() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;

            class C {}
            compact struct S {
                c: C
            }

            interface I {
                op(s: tag(1) S?);
            }
            ";
        let expected_errors = ["invalid type `s`: tagged members cannot contain classes"];

        // Act
        let errors = parse_for_errors(slice);

        // Assert
        assert_errors!(errors, expected_errors);
    }

    #[test]
    fn valid_tag() {
        // Arrange
        let slice = "
            module Test;
            struct S {
                a: tag(1) int32?
            }
            ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let data_member_ptr = ast.find_typed_entity::<DataMember>("Test::S::a").unwrap();
        let data_member_tag = data_member_ptr.borrow().tag();

        assert_eq!(data_member_tag, Some(1));
        assert!(data_member_ptr.borrow().data_type.is_optional);
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

        // Assert
        assert_errors!(error_reporter, [
            "invalid tag on member `b`: tags must be unique",
            "The data member `a` has previous used the tag value `1`",
        ]);
    }

    #[test]
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
        let expected_errors = [format!(
            "tag is out of range: {}. Tag values must be less than 2147483647",
            max_value + 1,
        )];

        // Act
        let error_reporter = parse_for_errors(&slice);

        // Assert
        assert_errors!(error_reporter, expected_errors);
    }

    #[test]
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
        let expected_errors = [format!("tag is out of range: {}. Tag values must be positive", -1)];

        // Act
        let error_reporter = parse_for_errors(&slice);

        // Assert
        assert_errors!(error_reporter, expected_errors);
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
