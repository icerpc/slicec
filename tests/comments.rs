// Copyright (c) ZeroC, Inc. All rights reserved.

mod helpers;

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

        {comment}
        interface MyInterface {{}}
        ",
        comment = doc_comment
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

#[test_case("/* This is a block comment. */")]
#[test_case("// This is a comment.")]
fn comments_are_ignored(comment: &str) {
    // Arrange
    let slice = &format!(
        "
        encoding = 2;
        module tests;

        {comment}
        interface MyInterface {{}}
        ",
        comment = comment
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
