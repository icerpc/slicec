// Copyright (c) ZeroC, Inc. All rights reserved.

mod container;
mod encoding;
mod inheritance;
mod tags;

use crate::helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;

#[test]
fn support_compact_type_id() {
    let slice = "
        encoding = 1;
        module Test;
        class C(42) {}
    ";

    let ast = parse_for_ast(slice);

    let class_ptr = ast.find_typed_type::<Class>("Test::C").unwrap();
    let class_def = class_ptr.borrow();

    assert_eq!(class_def.compact_id, Some(42));
}
