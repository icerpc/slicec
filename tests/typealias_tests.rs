// Copyright (c) ZeroC, Inc.

pub mod test_helpers;

mod typealias {

    use crate::test_helpers::*;
    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::*;
    use slice::slice_file::Span;
    use test_case::test_case;

    #[test_case("struct S {}", "S", 2 ; "structs")]
    #[test_case("exception E { }", "E", 2; "exceptions")]
    #[test_case("class C {}", "C", 1; "classes")]
    #[test_case("interface I {}", "I", 2; "interfaces")]
    #[test_case("enum E { Foo }", "E", 1; "enums")]
    #[test_case("custom C", "C", 2; "custom types")]
    #[test_case("", "bool", 2; "primitives")]
    #[test_case("", "sequence<bool>", 2; "sequences")]
    #[test_case("", "dictionary<bool, bool>", 2; "dictionaries")]
    #[test_case("typealias T = bool", "T", 2; "type aliases")]
    fn can_have_type_alias_of(definition: &str, identifier: &str, encoding: i32) {
        // Arrange
        let slice = format!(
            "
                encoding = {encoding}
                module Test
                {definition}
                typealias Alias = {identifier}
            "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let type_alias = ast.find_element::<TypeAlias>("Test::Alias").unwrap();
        type_alias.underlying.definition(); // Panics if the type-alias hasn't been initialized correctly.
    }

    #[test]
    fn can_be_used_as_field() {
        // Arrange
        let slice = "
            module Test
            typealias MyDict = dictionary<varint32, sequence<uint8>>
            compact struct S {
                dict: MyDict
            }
        ";

        // Act/Assert
        assert_parses(slice);
    }

    #[test]
    fn can_be_used_as_parameter() {
        // Arrange
        let slice = "
            module Test
            typealias MyDict = dictionary<varint32, sequence<uint8>>
            interface I {
                op(dict: MyDict)
            }
        ";

        // Act/Assert
        assert_parses(slice);
    }

    #[test]
    fn is_resolvable_as_an_entity() {
        // Arrange
        let slice = "
            module Test
            typealias MyInt = varuint32
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
            module Test
            typealias MyInt = varuint32
            compact struct S {
                a: MyInt
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::a").unwrap();

        assert_eq!(field.identifier(), "a");
        assert!(matches!(
            field.data_type.concrete_type(),
            Types::Primitive(Primitive::VarUInt32),
        ));
    }

    #[test]
    fn cannot_be_optional() {
        // Arrange
        let slice = "
            module Test
            typealias Test = bool?
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::TypeAliasOfOptional)
            .set_span(&Span::new((3, 13).into(), (3, 27).into(), "string-0"))
            .add_note(
                "try removing the trailing `?` modifier from its definition",
                Some(&Span::new((3, 30).into(), (3, 35).into(), "string-0")),
            )
            .add_note(
                "instead of aliasing an optional type directly, try making it optional where you use it",
                None,
            );

        check_diagnostics(diagnostics, [expected]);
    }
}
