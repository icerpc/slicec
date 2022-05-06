// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;

#[test]
fn can_contain_tags() {
    // Arrange
    let slice = "
        module Test;
        exception E {
            i: int32,
            s: string,
            b: tag(10) bool?,
        }
        ";
    let ast = parse_for_ast(slice);

    // Assert
    let data_member_ptr = ast.find_typed_entity::<DataMember>("Test::E::b").unwrap();
    let data_member_tag = data_member_ptr.borrow().tag();

    assert_eq!(data_member_tag, Some(10));
}
