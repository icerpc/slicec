// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

use crate::helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;

mod module {
    use super::*;

    #[test]
    #[ignore]
    fn can_be_reopened() {
        let slice = "
        module mod
        {
            struct S1 {}
        }

        module mod
        {
            struct S2 {}
        }
        ";

        let ast = parse_for_ast(slice);

        let module_ptr = ast.find_typed_entity::<Module>("mod").unwrap();
        let module_def = module_ptr.borrow();
        let contents = module_def.contents();

        assert_eq!(contents.len(), 2);
    }

    #[test]
    fn can_be_nested() {
        let slice = "
        module A
        {
            module B {}
        }
        ";

        let ast = parse_for_ast(slice);

        assert!(ast.find_typed_entity::<Module>("A::B").is_some());
    }

    #[test]
    fn can_use_nested_syntax() {
        let slice = "
        module A::B::C::D {}
        ";

        let ast = parse_for_ast(slice);

        assert!(ast.find_typed_entity::<Module>("A::B::C::D").is_some());
    }
}
