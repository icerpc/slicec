// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

use crate::helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;
use test_case::test_case;

#[test_case("/** This is a block doc comment. */", "This is a block doc comment.")]
#[test_case(
    "/**\n* This is a multi-line block doc comment.\n*/",
    "This is a multi-line block doc comment.\n"
)]
#[test_case("/// This is a doc comment.", "This is a doc comment.")]
#[ignore] // TODO: fix the parsing of block doc comments to remove /** */ from overview and remove \n from end
          // of doc comment.
fn doc_comments_added_to_comment_overview(doc_comment: &str, expected: &str) {
    // Arrange
    let slice = &format!(
        "
        encoding = 2;
        module tests;

        {}
        interface MyInterface {{}}
        ",
        doc_comment
    );

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let interface_ptr = ast
        .find_typed_type::<Interface>("tests::MyInterface")
        .unwrap();
    let interface_def = interface_ptr.borrow();
    let interface_doc = interface_def.comment().unwrap();

    assert_eq!(interface_doc.overview, expected);
}

#[test]
fn doc_comments_params() {
    // Arrange
    let slice = "
        encoding = 2;
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
    let op_ptr = ast
        .find_typed_entity::<Operation>("tests::TestInterface::testOp")
        .unwrap();
    let op_def = op_ptr.borrow();
    let op_doc_comment = op_def.comment().unwrap();

    assert_eq!(op_doc_comment.params, expected);
}

#[test]
fn doc_comments_returns() {
    // Arrange
    let slice = "
        encoding = 2;
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
    let op_ptr = ast
        .find_typed_entity::<Operation>("tests::TestInterface::testOp")
        .unwrap();
    let op_def = op_ptr.borrow();
    let op_doc_comment = op_def.comment().unwrap();

    assert_eq!(op_doc_comment.returns, expected);
}

#[test]
fn doc_comments_throws() {
    // Arrange
    let slice = "
        encoding = 2;
        module tests;

        interface TestInterface {
            /// @throws MyThrownThing Message about my thrown thing.
            testOp(testParam: string) -> bool;
        }
        ";
    let expected = vec![(
        "MyThrownThing".to_owned(),
        "Message about my thrown thing.".to_owned(),
    )];

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let op_ptr = ast
        .find_typed_entity::<Operation>("tests::TestInterface::testOp")
        .unwrap();
    let op_def = op_ptr.borrow();
    let op_doc_comment = op_def.comment().unwrap();

    assert_eq!(op_doc_comment.throws, expected);
}

#[test]
#[ignore]
fn doc_comments_see_also() {
    // Arrange
    let slice = "
        encoding = 2;
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
    let op_ptr = ast
        .find_typed_entity::<Operation>("tests::TestInterface::testOp")
        .unwrap();
    let op_def = op_ptr.borrow();
    let op_doc_comment = op_def.comment().unwrap();

    assert_eq!(op_doc_comment.see_also, expected);
}

#[test]
fn doc_comments_location() {
    // Arrange
    let slice = "
encoding = 2;
module tests;

/// This is a doc comment.
interface MyInterface {}
";
    let expected_start = (5, 1);
    let expected_end = (6, 1);

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let interface_ptr = ast
        .find_typed_type::<Interface>("tests::MyInterface")
        .unwrap();
    let interface_def = interface_ptr.borrow();
    let interface_doc = interface_def.comment().unwrap();

    assert_eq!(interface_doc.location.start, expected_start);
    assert_eq!(interface_doc.location.end, expected_end);
}

#[test_case("/* This is a block comment. */")]
#[test_case("/*\n* This is a multiline block comment.\n */")]
#[test_case("// This is a comment.")]
fn comments_are_ignored(comment: &str) {
    // Arrange
    let slice = &format!(
        "
        encoding = 2;
        module tests;

        {}
        interface MyInterface {{}}
        ",
        comment
    );

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let interface_ptr = ast
        .find_typed_type::<Interface>("tests::MyInterface")
        .unwrap();
    let interface_def = interface_ptr.borrow();
    let interface_doc = interface_def.comment();

    assert!(interface_doc.is_none());
}
