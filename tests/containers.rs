// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::Ast;
use slice::parse_from_string;

fn parse(slice: &str) -> Ast {
    let (ast, error_reporter) = parse_from_string(slice).ok().unwrap();
    assert!(!error_reporter.has_errors(true));
    ast
}

mod exceptions {
    use slice::grammar::*;
    use super::*;

    #[test]
    fn can_contain_data_members() {
        let ast = parse("
    module Test;
    exception E
    {
        i: int32,
        s: string,
        b: bool,
    }"
        );

        let struct_ptr = ast.find_typed_type::<Exception>("Test::E").unwrap();
        let struct_def = struct_ptr.borrow();
        let data_members = struct_def.members();

        assert_eq!(data_members.len(), 3);

        assert!(matches!(data_members[0].identifier(), "i"));
        assert!(matches!(data_members[1].identifier(), "s"));
        assert!(matches!(data_members[2].identifier(), "b"));

        assert!(matches!(data_members[0].data_type.concrete_type(),
            Types::Primitive(Primitive::Int32)));
        assert!(matches!(data_members[1].data_type.concrete_type(),
            Types::Primitive(Primitive::String)));
        assert!(matches!(data_members[2].data_type.concrete_type(),
            Types::Primitive(Primitive::Bool)));
    }

    #[test]
    fn can_be_empty() {
        let ast = parse("
            module Test;
            exception E {}
        ");

        let struct_ptr = ast.find_typed_type::<Exception>("Test::E").unwrap();
        let struct_def = struct_ptr.borrow();
        let data_members = struct_def.members();

        assert_eq!(data_members.len(), 0);
    }
}


mod structs {
    use slice::grammar::*;
    use super::*;

    #[test]
    fn can_contain_data_members() {
        let ast = parse("
    module Test;
    struct S
    {
        i: int32,
        s: string,
        b: bool,
    }"
        );

        let struct_ptr = ast.find_typed_type::<Struct>("Test::S").unwrap();
        let struct_def = struct_ptr.borrow();
        let data_members = struct_def.members();

        assert_eq!(data_members.len(), 3);

        assert!(matches!(data_members[0].identifier(), "i"));
        assert!(matches!(data_members[1].identifier(), "s"));
        assert!(matches!(data_members[2].identifier(), "b"));

        assert!(matches!(data_members[0].data_type.concrete_type(),
            Types::Primitive(Primitive::Int32)));
        assert!(matches!(data_members[1].data_type.concrete_type(),
            Types::Primitive(Primitive::String)));
        assert!(matches!(data_members[2].data_type.concrete_type(),
            Types::Primitive(Primitive::Bool)));
    }

    #[test]
    fn can_be_empty() {
        let ast = parse("
            module Test;
            struct S {}
        ");

        let struct_ptr = ast.find_typed_type::<Struct>("Test::S").unwrap();
        let struct_def = struct_ptr.borrow();
        let data_members = struct_def.members();

        assert_eq!(data_members.len(), 0);
    }
}

mod compact_structs {
    use slice::parse_from_string;

    #[test]
    #[ignore]
    fn must_not_be_empty() {
        let slice = "
            encoding = 2;
            module Test;
            compact struct S {}";
        let (_, error_reporter) = parse_from_string(slice).ok().unwrap();

        error_reporter.assert_errors(&["compact structs must be non-empty"]);
    }
}
