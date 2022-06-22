// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod module {

    use crate::helpers::parsing_helpers::parse_for_ast;
    use slice::grammar::*;
    use slice::parse_from_string;

    #[test]
    fn can_be_reopened() {
        let slice = "
        module Test
        {
            struct S1 {}
        }

        module Test
        {
            struct S2 {}
        }
        ";

        let ast = parse_for_ast(slice);

        assert!(ast.find_element::<Struct>("Test::S1").is_ok());
        assert!(ast.find_element::<Struct>("Test::S2").is_ok());
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

        assert!(ast.find_element::<Module>("A::B").is_ok());
    }

    #[test]
    fn can_use_nested_syntax() {
        let slice = "
        module A::B::C::D {}
        ";

        let ast = parse_for_ast(slice);

        assert!(ast.find_element::<Module>("A::B::C::D").is_ok());
    }

    #[test]
    fn is_required() {
        // TODO: better error message once we replace the parser
        let slice = "custom C;";
        let err = parse_from_string(slice).err();
        assert!(err.is_some());
    }
}
