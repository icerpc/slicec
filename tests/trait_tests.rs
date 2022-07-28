// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod traits {

    use crate::helpers::parsing_helpers::parse_for_ast;
    use slice::grammar::*;

    mod encoding {

        mod slice1 {

            use slice::errors::{ErrorKind, RuleKind};
            use slice::grammar::Encoding;

            use crate::assert_errors_new;
            use crate::helpers::parsing_helpers::parse_for_errors;

            #[test]
            fn are_not_supported() {
                // Arrange
                let slice = "
                    encoding = 1;
                    module Test;
                    trait ATrait;
                ";
                let expected = [
                    RuleKind::NotSupportedWithEncoding("trait".to_owned(), "ATrait".to_owned(), Encoding::Slice1)
                        .into(),
                    ErrorKind::new("file encoding was set to Slice1 here:".to_owned()),
                    ErrorKind::new("traits are not supported by the Slice1 encoding".to_owned()),
                ];

                // Act
                let error_reporter = parse_for_errors(slice);

                // Assert
                assert_errors_new!(error_reporter, expected);
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
        let trait_def = ast.find_element::<Trait>("Test::ATrait").unwrap();
        assert_eq!(trait_def.identifier(), "ATrait");
    }
}
