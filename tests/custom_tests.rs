// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

use helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;

#[test]
fn is_resolvable_as_an_entity() {
    let slice = "
    module Test;
    custom ACustomType;
    ";

    let ast = parse_for_ast(slice);

    let custom_ptr = ast
        .find_typed_entity::<CustomType>("Test::ACustomType")
        .unwrap();
    let custom = custom_ptr.borrow();

    assert_eq!(custom.identifier(), "ACustomType");
}
