// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod traits {

    use crate::helpers::parsing_helpers::parse_for_ast;
    use slice::grammar::*;

    mod encoding {

        mod slice1 {

            use slice::diagnostics::{Diagnostic, LogicErrorKind, Note};
            use slice::grammar::Encoding;

            use crate::assert_errors;
            use crate::helpers::parsing_helpers::parse_for_diagnostics;

            #[test]
            fn are_not_supported() {
                // Arrange
                let slice = "
                    encoding = 1;
                    module Test;
                    trait ATrait;
                ";

                // Act
                let diagnostic_reporter = parse_for_diagnostics(slice);

                // Assert
                let expected = Diagnostic::new_with_notes(
                    LogicErrorKind::NotSupportedWithEncoding("trait".to_owned(), "ATrait".to_owned(), Encoding::Slice1),
                    None,
                    vec![
                        Note::new("file encoding was set to Slice1 here:", None),
                        Note::new("traits are not supported by the Slice1 encoding", None),
                    ],
                );
                assert_errors!(diagnostic_reporter, [&expected]);
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
