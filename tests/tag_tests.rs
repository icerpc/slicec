// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

use crate::helpers::parsing_helpers::parse_for_errors;

mod tags {

    use super::*;
    use test_case::test_case;

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
                b: tag(10) B,
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
            op(a: tag(1) C);
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
}
