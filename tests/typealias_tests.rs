// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

use helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;

mod typealias {

    use super::*;

    #[test]
    fn is_resolvable_as_an_entity() {
        let slice = "
        module Test;
        typealias MyInt = varuint32;
        ";

        let ast = parse_for_ast(slice);

        let type_alias_ptr = ast.find_typed_entity::<TypeAlias>("Test::MyInt").unwrap();
        let type_alias = type_alias_ptr.borrow();

        assert_eq!(type_alias.identifier(), "MyInt");
        assert!(matches!(
            type_alias.underlying.concrete_type(),
            Types::Primitive(Primitive::VarUInt32)
        ));
    }

    #[test]
    fn is_resolved_as_the_aliased_type_when_used() {
        let slice = "
        module Test;
        typealias MyInt = varuint32;
        compact struct S
        {
            a: MyInt,
        }
        ";

        let ast = parse_for_ast(slice);

        let data_member_ptr = ast.find_typed_entity::<DataMember>("Test::S::a").unwrap();
        let data_member = data_member_ptr.borrow();

        assert_eq!(data_member.identifier(), "a");
        assert!(matches!(
            data_member.data_type.concrete_type(),
            Types::Primitive(Primitive::VarUInt32)
        ));
    }
}
