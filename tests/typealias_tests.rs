// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod typealias {

    use crate::helpers::parsing_helpers::parse_for_ast;
    use slice::grammar::*;

    #[test]
    fn is_resolvable_as_an_entity() {
        // Arrange
        let slice = "
            module Test;
            typealias MyInt = varuint32;
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let type_alias = ast.find_element::<TypeAlias>("Test::MyInt").unwrap();
        assert_eq!(type_alias.identifier(), "MyInt");
        assert!(matches!(
            type_alias.underlying.concrete_type(),
            Types::Primitive(Primitive::VarUInt32),
        ));
    }

    #[test]
    fn is_resolved_as_the_aliased_type_when_used() {
        // Arrange
        let slice = "
            module Test;
            typealias MyInt = varuint32;
            compact struct S
            {
                a: MyInt,
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let data_member = ast.find_element::<DataMember>("Test::S::a").unwrap();
        assert_eq!(data_member.identifier(), "a");
        assert!(matches!(
            data_member.data_type.concrete_type(),
            Types::Primitive(Primitive::VarUInt32),
        ));
    }
}
