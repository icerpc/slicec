// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod custom {

    mod slice1 {

        use crate::assert_errors;
        use crate::helpers::parsing_helpers::parse_for_errors;

        #[test]
        fn is_not_supported() {
            // Arrange
            let slice = "
            encoding = 1;
            module Test;
            custom ACustomType;
            ";

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            assert_errors!(error_reporter, &[
                "custom types are not supported by the Slice 1 encoding",
                "file encoding was set to the Slice 1 encoding here:",
            ]);
        }
    }

    mod slice2 {

        use crate::assert_errors;
        use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_errors};
        use slice::grammar::*;

        #[test]
        fn is_valid() {
            // Arrange
            let slice = "
            module Test;
            custom ACustomType;
            ";

            // Act
            let errors = parse_for_errors(slice);

            // Assert
            assert_errors!(errors);
        }

        #[test]
        fn is_resolvable_as_an_entity() {
            // Arrange
            let slice = "
            module Test;
            custom ACustomType;
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let custom_ptr = ast
                .find_typed_entity::<CustomType>("Test::ACustomType")
                .unwrap();
            let custom = custom_ptr.borrow();

            assert_eq!(custom.identifier(), "ACustomType");
        }
    }
}
