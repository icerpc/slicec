// Copyright (c) ZeroC, Inc.

use crate::helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;

#[test]
fn can_contain_tags() {
    // Arrange
    let slice = "
        encoding = 1;
        module Test;

        class C {
            i: int32,
            s: string,
            b: tag(10) bool?,
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let data_member = ast.find_element::<DataMember>("Test::C::b").unwrap();
    assert_eq!(data_member.tag(), Some(10));
}
