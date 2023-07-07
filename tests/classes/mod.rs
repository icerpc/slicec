// Copyright (c) ZeroC, Inc.

mod container;
mod inheritance;
mod mode;
mod tags;

use crate::test_helpers::*;
use slicec::grammar::*;

#[test]
fn support_compact_type_id() {
    // Arrange
    let slice = "
        mode = Slice1
        module Test

        class C(42) {}
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let class_def = ast.find_element::<Class>("Test::C").unwrap();
    let compact_id = class_def.compact_id.as_ref().unwrap();
    assert_eq!(compact_id.value, 42);
}
