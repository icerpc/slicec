// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod traits {

    use crate::helpers::parsing_helpers::parse_for_ast;
    use slice::grammar::*;

    mod encoding {

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
                assert_errors!(error_reporter, [
                    "traits are not supported by the Slice1 encoding",
                    "file encoding was set to the Slice1 encoding here:",
                ]);
            }
        }
    }

    #[test]
    fn type_parses() {
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
