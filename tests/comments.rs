// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

use crate::helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;
use slice::slice_file::Location;
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

#[test_case(
    "/** @param testParam My test param */",
    vec![("testParam".to_owned(), "My test param".to_owned())];
    "MyTest"
)]
#[test_case(
    "/// @param testParam My test param",
    vec![("testParam".to_owned(),
    "My test param".to_owned())];
    "MyTest2"
)]
#[ignore] // TODO: /** test case fails as the params are never parsed, /// fails because of a trailing \n
fn doc_comments_params(doc_comment: &str, expected: Vec<(String, String)>) {
    // Arrange
    let slice = &format!(
        "
        encoding = 2;
        module tests;

        interface TestInterface {{
            {}
            testOp(testParam: string);
        }}
        ",
        doc_comment
    );

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

#[test_case(
    "/** @return bool*/",
    Some("bool".to_owned());
    "block comment"
)] // TODO: fix, star stripping
#[test_case(
    "/// @return bool",
    Some("bool".to_owned());
    "comment")] // TODO: fix trailing \n
#[ignore]
fn doc_comments_returns(doc_comment: &str, expected: Option<String>) {
    // Arrange
    let slice = &format!(
        "
        encoding = 2;
        module tests;

        interface TestInterface {{
            {}
            testOp(testParam: string) -> bool;
        }}
        ",
        doc_comment
    );

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

#[test_case(
    "/** @throws MyThrownThing Message about my thrown thing.*/",
    vec![("MyThrownThing".to_owned(), "Message about my thrown thing.".to_owned())];
    "block comment"
)] // TODO: Fix star stripping
#[test_case(
    "/// @throws MyThrownThing Message about my thrown thing.",
    vec![("MyThrownThing".to_owned(), "Message about my thrown thing.".to_owned())];
    "comment"
)] // TODO: Fix trailing \n
#[ignore]
fn doc_comments_throws(doc_comment: &str, expected: Vec<(String, String)>) {
    // Arrange
    let slice = &format!(
        "
        encoding = 2;
        module tests;

        interface TestInterface {{
            {}
            testOp(testParam: string) -> bool;
        }}
        ",
        doc_comment
    );

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

#[test_case(
    "/** @see MySee Message about thing.*/",
    vec!["MySee".to_owned()];
    "block comment"
)] // TODO: Fix star stripping
#[test_case(
    "/// @see MySee Message about thing.",
    vec!["MySee".to_owned()];
    "comment"
)]
#[ignore]
fn doc_comments_see_also(doc_comment: &str, expected: Vec<String>) {
    // Arrange
    let slice = &format!(
        "
        encoding = 2;
        module tests;

        interface TestInterface {{
            {}
            testOp(testParam: string) -> bool;
        }}
        ",
        doc_comment
    );

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

#[test_case("/** This is a block doc comment. */", (5, 36))]
#[test_case("/**\n* This is a multi-line block doc comment.\n*/", (7, 3))]
#[test_case("/// This is a doc comment.", (6, 1))]
#[ignore] // TODO: When fixing updated expected_end, currently influenced by the \n or lack of star stripping
fn doc_comments_location(comment: &str, expected_end: (usize, usize)) {
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
    let interface_doc = interface_def.comment().unwrap();

    assert_eq!(interface_doc.location.start, (5, 1));
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
