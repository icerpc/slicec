// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod traits {

    mod slice1 {

        use crate::assert_errors;
        use crate::helpers::parsing_helpers::parse_for_errors;

        #[test]
        fn are_not_supported() {
            // Arrange
            let slice = "
            encoding = 1;
            module Test;
            trait ATrait;
            ";

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            assert_errors!(error_reporter, &[
                "traits are not supported by the Slice 1 encoding",
                "file encoding was set to the Slice 1 encoding here:",
            ]);
        }
    }

    mod slice2 {

        use crate::assert_errors;
        use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_errors};
        use slice::grammar::*;

        #[test]
        fn are_valid() {
            // Arrange
            let slice = "
            module Test;
            trait ATrait;
            ";

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            assert_errors!(error_reporter);
        }

        #[test]
        fn are_resolvable_as_an_entity() {
            // Arrange
            let slice = "
            module Test;
            trait ATrait;
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let trait_ptr = ast.find_typed_entity::<Trait>("Test::ATrait").unwrap();
            let trait_def = trait_ptr.borrow();

            assert_eq!(trait_def.identifier(), "ATrait");
        }
    }
}
