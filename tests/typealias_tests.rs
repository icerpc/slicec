// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod typealias {

    use crate::helpers::parsing_helpers::parse_for_ast;
    use slice::grammar::*;
    use slice::parse_from_string;
    use test_case::test_case;

    #[test_case("struct S {}",     "S"; "structs")]
    #[test_case("exception E { }", "E"; "exceptions")]
    #[test_case("class C {}",      "C"; "classes")]
    #[test_case("interface I {}",  "I"; "interfaces")]
    #[test_case("enum E { Foo }",  "E"; "enums")]
    #[test_case("trait T;",        "T"; "traits")]
    #[test_case("custom C;",       "C"; "custom types")]
    #[test_case("", "bool"; "primitives")]
    #[test_case("", "sequence<bool>"; "sequences")]
    #[test_case("", "dictionary<bool, bool>"; "dictionaries")]
    #[test_case("typealias T = bool;", "T"; "type aliases")]
    fn can_have_type_alias_of(definition: &str, identifier: &str) {
        // Arrange
        let slice = format!(
            "
                encoding = {};
                module Test;
                {}
                typealias Alias = {};
            ",
            if definition == "class C {}" { 1 } else { 2 },
            definition,
            identifier,
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let type_alias = ast.find_element::<TypeAlias>("Test::Alias").unwrap();
        assert!(type_alias.underlying.definition.is_initialized());
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
        let result = parse_from_string(slice);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn can_be_used_as_parameter() {
        // Arrange
        let slice = "
            module Test;
            typealias MyDict = dictionary<varint32, sequence<uint8>>;
            interface I {
                op(dict: MyDict);
            }
        ";

        // Act
        let result = parse_from_string(slice);

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
