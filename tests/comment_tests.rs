// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod comments {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_errors};
    use slice::grammar::*;
    use test_case::test_case;

    #[test_case("/** This is a block doc comment. */", "This is a block doc comment."; "block doc comment")]
    #[test_case(
        "/**\n* This is a multi-line block doc comment.\n*/",
        "This is a multi-line block doc comment.\n"
        ; "multi-line block doc comment"
    )]
    #[test_case("/// This is a doc comment.", "This is a doc comment."; "doc comment")]
    #[ignore] // TODO: fix the parsing of block doc comments to remove /** */ from overview and remove \n from
              // end of doc comment.
    fn doc_comments_added_to_comment_overview(doc_comment: &str, expected: &str) {
        // Arrange
        let slice = format!(
            "
            module tests;

            {}
            interface MyInterface {{}}
            ",
            doc_comment
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
        let expected = vec![("testParam".to_owned(), "My test param".to_owned())];

        // Act
        let ast = parse_for_ast(slice);

        // Assert
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
        let expected = Some("bool".to_owned());

        // Act
        let ast = parse_for_ast(slice);

        // Assert
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
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, [
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
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, [
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
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter);
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
        let expected_throws = vec![(
            "MyThrownThing".to_owned(),
            "Message about my thrown thing.\nMore about the thrown thing.".to_owned(),
        )];

        // Act
        let ast = parse_for_ast(slice);

        // Assert
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
        let expected = vec![("MyThrownThing".to_owned(), "Message about my thrown thing.".to_owned())];

        // Act
        let ast = parse_for_ast(slice);

        // Assert
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
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, [
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
        let expected = vec!["MySee".to_owned()];

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let operation = ast.find_element::<Operation>("tests::TestInterface::testOp").unwrap();
        let op_doc_comment = operation.comment().unwrap();

        assert_eq!(op_doc_comment.see_also, expected);
    }

    #[test_case("/// This is a doc comment.", (4, 13), (5, 13); "doc comment")]
    #[test_case("/**\n* This is a multi line doc comment.\n*/", (4, 13), (6, 3); "multi-line doc comment")]
    fn doc_comments_location(comment: &str, expected_start: (usize, usize), expected_end: (usize, usize)) {
        // Arrange
        let slice = format!(
            "
            module tests;

            {}
            interface MyInterface {{}}
            ",
            comment,
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let interface_def = ast.find_element::<Interface>("tests::MyInterface").unwrap();
        let interface_doc = interface_def.comment().unwrap();

        assert_eq!(interface_doc.location.start, expected_start);
        assert_eq!(interface_doc.location.end, expected_end);
    }

    #[test_case("/* This is a block comment. */"; "block comment")]
    #[test_case("/*\n* This is a multiline block comment.\n */"; "multi-line block comment")]
    #[test_case("// This is a comment."; "comment")]
    fn non_doc_comments_are_ignored(comment: &str) {
        // Arrange
        let slice = format!(
            "
            module tests;

            {}
            interface MyInterface {{}}
            ",
            comment,
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let interface_def = ast.find_element::<Interface>("tests::MyInterface").unwrap();
        let interface_doc = interface_def.comment();

        assert!(interface_doc.is_none());
    }
}
