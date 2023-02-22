// Copyright (c) ZeroC, Inc.

pub mod helpers;

mod module {

    use crate::helpers::parsing_helpers::*;
    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::*;
    use test_case::test_case;

    #[test_case("{}", false; "normal")]
    #[test_case(";", true; "file_scoped")]
    fn can_be_defined(content: &str, expected: bool) {
        // Arrange
        let slice = format!("module Test {content}");

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        assert_eq!(ast.find_element::<Module>("Test").unwrap().is_file_scoped, expected);
    }

    #[test]
    fn can_be_reopened() {
        // Arrange
        let slice = "
            module Test {
                struct S1 {}
            }

            module Test {
                struct S2 {}
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        assert!(ast.find_element::<Struct>("Test::S1").is_ok());
        assert!(ast.find_element::<Struct>("Test::S2").is_ok());
    }

    #[test]
    fn can_be_nested() {
        // Arrange
        let slice = "
            module A {
                module B {}
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        assert!(ast.find_element::<Module>("A::B").is_ok());
    }

    #[test]
    fn can_use_nested_syntax() {
        // Arrange
        let slice = "
            module A::B::C::D {}
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        assert!(ast.find_element::<Module>("A::B::C::D").is_ok());
    }

    #[test]
    fn is_required() {
        // Arrange
        let slice = "
            custom C;
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::Syntax {
            message: "expected one of '[', '[[', 'doc comment', 'encoding', or 'module', but found 'custom'".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn file_level_modules_can_not_contain_sub_modules() {
        // Arrange
        let slice = "
            module A;

            module B {}

            module C {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = [
            Error::new(ErrorKind::FileScopedModuleCannotContainSubModules {
                identifier: "A".to_owned(),
            }),
            Error::new(ErrorKind::FileScopedModuleCannotContainSubModules {
                identifier: "A".to_owned(),
            }),
        ];
        check_diagnostics(diagnostics, expected);
    }

    #[test]
    fn nested_file_level_modules_can_not_contain_sub_modules() {
        // Arrange
        let slice = "
            module A::B::C::D;

            module E {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = [Error::new(ErrorKind::FileScopedModuleCannotContainSubModules {
            identifier: "D".to_owned(),
        })];
        check_diagnostics(diagnostics, expected);
    }

    #[test]
    fn cross_module_redefinitions_are_disallowed() {
        // Arrange
        let slice = "
            module Foo {
                struct Bar {}
            }

            module Foo {
                struct Bar {}
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::Redefinition {
            identifier: "Bar".to_owned(),
        })
        .add_note("'Bar' was previously defined here", None);

        check_diagnostics(diagnostics, [expected]);
    }

    #[test_case("Foo"; "module")]
    #[test_case("Foo::Bar"; "nested module")]
    fn modules_can_be_reopened(module_name: &str) {
        // Arrange
        let slice = format!(
            "
            module {module_name} {{}}
            module {module_name} {{}}
            "
        );

        // Act/Assert
        parse_for_ast(slice);
    }
}
