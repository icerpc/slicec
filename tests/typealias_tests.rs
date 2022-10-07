// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod typealias {

    use crate::helpers::parsing_helpers::parse_for_ast;
    use slice::grammar::*;
    use slice::parse_from_strings;
    use test_case::test_case;

    #[test_case("struct S {}", "S", 2 ; "structs")]
    #[test_case("exception E { }", "E", 2; "exceptions")]
    #[test_case("class C {}", "C", 1; "classes")]
    #[test_case("interface I {}", "I", 2; "interfaces")]
    #[test_case("enum E { Foo }", "E", 2; "enums")]
    #[test_case("trait T;", "T", 2; "traits")]
    #[test_case("custom C;", "C", 2; "custom types")]
    #[test_case("", "bool", 2; "primitives")]
    #[test_case("", "sequence<bool>", 2; "sequences")]
    #[test_case("", "dictionary<bool, bool>", 2; "dictionaries")]
    #[test_case("typealias T = bool;", "T", 2; "type aliases")]
    fn can_have_type_alias_of(definition: &str, identifier: &str, encoding: i32) {
        // Arrange
        let slice = format!(
            "
                encoding = {encoding};
                module Test;
                {definition}
                typealias Alias = {identifier};
            "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let type_alias = ast.find_element::<TypeAlias>("Test::Alias").unwrap();
        type_alias.underlying.definition(); // Panics if the type-alias hasn't been initialized correctly.
    }

    #[test]
    fn can_be_used_as_data_member() {
        // Arrange
        let slice = "
            module Test;
            typealias MyDict = dictionary<varint32, sequence<uint8>>;
            compact struct S
            {
                dict: MyDict,
            }
        ";

        // Act
        let result = parse_from_strings(&[slice], None);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn can_be_used_as_parameter() {
        // Arrange
        let slice = "
            module Test;
            typealias MyDict = dictionary<varint32, sequence<uint8>>;
            interface I
            {
                op(dict: MyDict);
            }
        ";

        // Act
        let result = parse_from_strings(&[slice], None);

        // Assert
        assert!(result.is_ok());
    }

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
