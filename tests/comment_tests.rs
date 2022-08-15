// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod comments {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_diagnostics};
    use slice::grammar::*;
    use test_case::test_case;

    #[test_case("/** This is a block doc comment. */", "This is a block doc comment."; "block doc comment")]
    #[test_case("/// This is a doc comment.", "This is a doc comment."; "doc comment")]
    #[test_case("/// This is a\n/// multiline doc comment.", "This is a\nmultiline doc comment."; "multiline doc comment")]
    #[test_case(
        "/**\n
        * This is a multi-line block doc comment.\n
        */",
        "This is a multi-line block doc comment."
        => ignore["reason"];
        "multi-line block doc comment"
    )] // TODO: Multi-line block doc comments parsing needs to be fixed to properly support multi-line block doc comments.
    fn doc_comments_added_to_comment_overview(doc_comment: &str, expected: &str) {
        // Arrange
        let slice = format!(
            "
                module tests;

                {doc_comment}
                interface MyInterface {{}}
            ",
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let interface_def = ast.find_element::<Interface>("tests::MyInterface").unwrap();
        let interface_doc = interface_def.comment().unwrap();

        assert_eq!(interface_doc.overview, expected);
    }

    #[test]
    fn doc_comments_params() {
        // Arrange
        let slice = "
            module tests;

            interface TestInterface {
                /// @param testParam My test param
                testOp(testParam: string);
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let expected = vec![("testParam".to_owned(), "My test param".to_owned())];
        let operation = ast.find_element::<Operation>("tests::TestInterface::testOp").unwrap();
        let op_doc_comment = operation.comment().unwrap();

        assert_eq!(op_doc_comment.params, expected);
    }

    #[test]
    fn doc_comments_returns() {
        // Arrange
        let slice = "
            module tests;

            interface TestInterface {
                /// @return bool
                testOp(testParam: string) -> bool;
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let expected = Some("bool".to_owned());
        let operation = ast.find_element::<Operation>("tests::TestInterface::testOp").unwrap();
        let op_doc_comment = operation.comment().unwrap();

        assert_eq!(op_doc_comment.returns, expected);
    }

    #[test]
    fn operation_with_no_return_but_doc_comment_contains_return_fails() {
        // Arrange
        let slice = "
            module tests;

            interface TestInterface {
                /// @return This operation will return a bool.
                testOp(testParam: string);
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(diagnostic_reporter, [
            "void operation must not contain doc comment return tag"
        ]);
    }

    #[test]
    fn operation_with_doc_comment_for_param_but_no_param_fails() {
        // Arrange
        let slice = "
            module tests;

            interface TestInterface {
                /// @param testParam1 A string param
                /// @param testParam2 A bool param
                testOp(testParam1: string);
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(diagnostic_reporter, [
            "doc comment has a param tag for 'testParam2', but there is no parameter by that name",
        ]);
    }

    #[test]
    fn operation_with_correct_doc_comments() {
        // Arrange
        let slice = "
            module tests;

            interface TestInterface {
                /// @param testParam1 A string param
                /// @return bool
                /// @throws MyException Some message about why testOp throws
                testOp(testParam1: string) -> bool;
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(diagnostic_reporter);
    }

    #[test]
    #[ignore] // TODO: fix star parsing, causing doc comment return message to be parsed incorrectly
    fn multiline_tag_comment() {
        // Arrange
        let slice = "
            module tests;

            interface TestInterface {
                /**
                 * @throws MyThrownThing Message about my thrown thing. \n More about the thrown thing.
                 * @return bool
                 */
                testOp(testParam: string) -> bool;
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let expected_throws = vec![(
            "MyThrownThing".to_owned(),
            "Message about my thrown thing.\nMore about the thrown thing.".to_owned(),
        )];
        let operation = ast.find_element::<Operation>("tests::TestInterface::testOp").unwrap();
        let op_doc_comment = operation.comment().unwrap();

        assert_eq!(op_doc_comment.throws, expected_throws);
        assert_eq!(op_doc_comment.returns, Some("bool\n".to_owned()));
    }

    #[test]
    fn doc_comments_throws() {
        // Arrange
        let slice = "
            module tests;

            interface TestInterface {
                /// @throws MyThrownThing Message about my thrown thing.
                testOp(testParam: string) -> bool;
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let expected = vec![("MyThrownThing".to_owned(), "Message about my thrown thing.".to_owned())];
        let operation = ast.find_element::<Operation>("tests::TestInterface::testOp").unwrap();
        let op_doc_comment = operation.comment().unwrap();

        assert_eq!(op_doc_comment.throws, expected);
    }

    #[test]
    fn doc_comments_non_operations_cannot_throw() {
        // Arrange
        let slice = "
            module tests;

            /// @throws MyThrownThing Message about my thrown thing.
            struct S {}
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(diagnostic_reporter, [
            "doc comment indicates that struct `S` throws, however, only operations can throw",
        ]);
    }

    #[test]
    #[ignore]
    fn doc_comments_see_also() {
        // Arrange
        let slice = "
            module tests;

            interface TestInterface {
                /// @see MySee Message about thing.
                testOp(testParam: string) -> bool;
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let expected = vec!["MySee".to_owned()];
        let operation = ast.find_element::<Operation>("tests::TestInterface::testOp").unwrap();
        let op_doc_comment = operation.comment().unwrap();

        assert_eq!(op_doc_comment.see_also, expected);
    }

    #[test_case("/// This is a doc comment.", (4, 13), (5, 13); "doc comment")]
    #[test_case("/**\n* This is a multi line doc comment.\n*/", (4, 13), (6, 3); "multi-line doc comment")]
    fn doc_comments_span(comment: &str, expected_start: (usize, usize), expected_end: (usize, usize)) {
        // Arrange
        let slice = format!(
            "
            module tests;

            {comment}
            interface MyInterface {{}}
            "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let interface_def = ast.find_element::<Interface>("tests::MyInterface").unwrap();
        let interface_doc = interface_def.comment().unwrap();

        assert_eq!(interface_doc.span.start, expected_start);
        assert_eq!(interface_doc.span.end, expected_end);
    }

    #[test_case("/* This is a block comment. */"; "block comment")]
    #[test_case("/*\n* This is a multiline block comment.\n */"; "multi-line block comment")]
    #[test_case("// This is a comment."; "comment")]
    fn non_doc_comments_are_ignored(comment: &str) {
        // Arrange
        let slice = format!(
            "
                module tests;

                {comment}
                interface MyInterface {{}}
            "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let interface_def = ast.find_element::<Interface>("tests::MyInterface").unwrap();
        let interface_doc = interface_def.comment();

        assert!(interface_doc.is_none());
    }
}
