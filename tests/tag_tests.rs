// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod tags {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_errors};
    use slice::grammar::*;
    use slice::parse_from_string;

    use test_case::test_case;

    #[test]
    #[ignore] // TODO: We do not verify that tagged data members are optional.
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
        assert_errors!(error_reporter, &["Tagged data members must be optional"]);
    }

    #[test]
    #[ignore] // TODO: Add error messages explaining that you cannot have tags on classes.
    fn cannot_tag_a_class() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;

            class B {
                i: int32,
            }
            class C {
                i: int32,
                s: string,
                b: tag(10) B?,
            }
            ";
        let expected_errors = [""];

        // Act
        let errors = parse_for_errors(slice);

        // Assert
        assert_errors!(errors, expected_errors);
    }

    #[test_case(
        "
        class A {
            a: tag(1) A,
        }"
    )]
    #[test_case(
        "
        class C {}
        interface I {
            op(a: tag(1) C?);
        }"
    )]
    #[ignore]
    fn cannot_tag_a_container_that_contains_a_class(slice_component: &str) {
        // Arrange
        let slice = format!(
            "
            encoding = 1;
            module Test;

            {}
            ",
            slice_component
        );
        let expected_errors = [""]; // TODO: Add error messages explaining that you cannot have tags on containers that contain
                                    // classes.

        // Act
        let errors = parse_for_errors(&slice);

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
    #[ignore] // TODO: Add error messages explaining that you cannot have multiple tags with the same value.
    fn cannot_have_duplicate_tags() {
        // Arrange
        let slice = "
            module Test;
            struct S {
                a: tag(1) int32?,
                b: tag(1) int32?,
            }
        ";
        let expected_errors = [""];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, expected_errors);
    }

    #[test_case(i32::MAX as i64, "2"; "Slice2 max value")]
    #[test_case(i32::MAX as i64, "1"; "Slice1 max value")]
    #[ignore] // TODO: Add error messages
    fn cannot_have_tag_with_value_larger_than_max(max: i64, encoding: &str) {
        // Arrange
        let slice = format!(
            "
            encoding = {encoding};
            module Test;
            interface I {{
                testOp(a: tag({max_value}) int32?);
            }}
        ",
            max_value = max + 1,
            encoding = encoding
        );
        let expected_errors = [""]; // TODO: Add error messages

        // Act
        let error_reporter = parse_for_errors(&slice);

        // Assert
        assert_errors!(error_reporter, expected_errors);
    }

    #[test_case(i32::MIN as i64, "2"; "Slice2 min value")]
    #[test_case(0, "1"; "Slice1 min value")]
    #[ignore] // TODO: Add error messages
    fn cannot_have_tag_with_value_smaller_than_minimum(min: i64, encoding: &str) {
        // Arrange
        let slice = format!(
            "
            encoding = {encoding};
            module Test;
            interface I {{
                testOp(a: tag({max_value}) int32?);
            }}
            ",
            max_value = min - 1,
            encoding = encoding
        );
        let expected_errors = [""]; // TODO: Add error messages

        // Act
        let error_reporter = parse_for_errors(&slice);

        // Assert
        assert_errors!(error_reporter, expected_errors);
    }

    #[test] // TODO: We should not be panicing here. We should be returning an error.
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

    #[test]
    #[ignore] // TODO: Add error messages
    fn negative_tags_are_invalid_with_slice1() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            interface I {
                testOp(a: tag(-1) int32?);
            }
        ";

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, &["Tags cannot be negative"]);
    }
}
