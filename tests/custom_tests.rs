// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod custom {

    use crate::helpers::parsing_helpers::parse_for_ast;
    use slice::grammar::*;

    mod encoding {

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
                assert_errors!(error_reporter, [
                    "custom types are not supported with Slice1",
                    "file encoding was set to Slice1 here:",
                ]);
            }
        }
    }

    #[test]
    fn type_parses() {
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
