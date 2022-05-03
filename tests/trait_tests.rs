// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

use helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;

#[test]
fn is_resolvable_as_an_entity() {
    let slice = "
    module Test;
    trait ATrait;
    ";

    let ast = parse_for_ast(slice);

    let trait_ptr = ast.find_typed_entity::<Trait>("Test::ATrait").unwrap();
    let trait_def = trait_ptr.borrow();

    assert_eq!(trait_def.identifier(), "ATrait");
}
