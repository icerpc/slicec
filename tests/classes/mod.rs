// Copyright (c) ZeroC, Inc. All rights reserved.

mod container;
mod encoding;
mod inheritance;
mod tags;

use crate::helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;

#[test]
fn support_compact_type_id() {
    // Arrange
    let slice = "
        encoding = 1;
        module Test;
        class C(42) {}
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let class_def = ast.find_element::<Class>("Test::C").unwrap();
    assert_eq!(class_def.compact_id, Some(42));
}
