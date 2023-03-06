// Copyright (c) ZeroC, Inc.

use crate::test_helpers::parse_for_ast;
use slice::grammar::*;

#[test]
fn can_contain_tags() {
    // Arrange
    let slice = "
        module Test
        exception E {
            i: int32
            s: string
            b: tag(10) bool?
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let data_member = ast.find_element::<DataMember>("Test::E::b").unwrap();
    assert_eq!(data_member.tag(), Some(10));
}
