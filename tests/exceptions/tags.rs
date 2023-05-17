// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slice::grammar::*;

#[test]
fn can_contain_tags() {
    // Arrange
    let slice = "
        module Test
        exception E {
            i: int32
            s: string
            tag(10) b: bool?
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let field = ast.find_element::<Field>("Test::E::b").unwrap();
    assert_eq!(field.tag(), Some(10));
}
