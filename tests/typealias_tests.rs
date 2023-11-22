// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod typealias {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use slicec::grammar::*;
    use slicec::slice_file::Span;
    use test_case::test_case;

    #[test_case("struct S {}", "S", "Slice2"; "structs")]
    #[test_case("class C {}", "C", "Slice1"; "classes")]
    #[test_case("enum E { Foo }", "E", "Slice1"; "enums")]
    #[test_case("custom C", "C", "Slice2"; "custom types")]
    #[test_case("", "bool", "Slice2"; "primitives")]
    #[test_case("", "Sequence<bool>", "Slice2"; "sequences")]
    #[test_case("", "Dictionary<bool, bool>", "Slice2"; "dictionaries")]
    #[test_case("typealias T = bool", "T", "Slice2"; "type aliases")]
    fn can_have_type_alias_of(definition: &str, identifier: &str, mode: &str) {
        // Arrange
        let slice = format!(
            "
                mode = {mode}
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
            typealias MyDict = Dictionary<varint32, Sequence<uint8>>
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
            typealias MyDict = Dictionary<varint32, Sequence<uint8>>
            interface I {
                op(dict: MyDict)
            }
        ";

        // Act/Assert
        assert_parses(slice);
    }

    #[test]
    fn is_resolvable() {
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
        let expected = Diagnostic::new(Error::TypeAliasOfOptional)
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

    #[test_case("Slice1", "uint32"; "Slice1")]
    #[test_case("Slice2", "AnyClass"; "Slice2")]
    fn reject_underlying_types_based_on_mode(mode: &str, underlying_type: &str) {
        // Arrange
        let slice = format!(
            "
            mode = {mode}
            module Test
            typealias Foo = {underlying_type}
            "
        );

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::UnsupportedType {
            kind: underlying_type.to_owned(),
            mode: match mode {
                "Slice1" => CompilationMode::Slice1,
                "Slice2" => CompilationMode::Slice2,
                _ => panic!(),
            },
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test_case("Slice1", "AnyClass"; "Slice1")]
    #[test_case("Slice2", "uint32"; "Slice2")]
    fn allow_underlying_types_based_on_mode(mode: &str, underlying_type: &str) {
        // Arrange
        let slice = format!(
            "
            mode = {mode}
            module Test
            typealias Foo = {underlying_type}
            "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let type_alias = ast.find_element::<TypeAlias>("Test::Foo").unwrap();
        assert_eq!(type_alias.underlying.type_string(), underlying_type);
    }
}
