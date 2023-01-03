// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod module {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_diagnostics};
    use slice::diagnostics::{Diagnostic, Error, ErrorKind};
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
            module Test
            {
                struct S1
                {
                }
            }

            module Test
            {
                struct S2
                {
                }
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
            module A
            {
                module B
                {
                }
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
            module A::B::C::D
            {
            }
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
        let reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(reporter, [
            "expected one of '[', '[[', 'doc comment', 'encoding', 'module', but found 'custom'"
        ]);
    }

    #[test]
    fn file_level_modules_can_not_contain_sub_modules() {
        // Arrange
        let slice = "
            module A;

            module B
            {
            }

            module C
            {
            }
        ";

        // Act
        let errors = parse_for_diagnostics(slice);

        // Assert
        let expected = vec![
            Error::new(ErrorKind::FileScopedModuleCannotContainSubModules {
                identifier: "A".to_owned(),
            }),
            Error::new(ErrorKind::FileScopedModuleCannotContainSubModules {
                identifier: "A".to_owned(),
            }),
        ];
        assert_errors!(errors, expected);
    }

    #[test]
    fn nested_file_level_modules_can_not_contain_sub_modules() {
        // Arrange
        let slice = "
            module A::B::C::D;

            module E
            {
            }
        ";

        // Act
        let errors = parse_for_diagnostics(slice);

        // Assert
        let expected = vec![Error::new(ErrorKind::FileScopedModuleCannotContainSubModules {
            identifier: "D".to_owned(),
        })];
        assert_errors!(errors, expected);
    }

    #[test]
    fn cross_module_redefinitions_are_disallowed() {
        // Arrange
        let slice = "
            module Foo
            {
                struct Bar {}
            }

            module Foo
            {
                struct Bar {}
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::Redefinition {
            identifier: "Bar".to_owned(),
        })
        .add_note("`Bar` was previously defined here", None);
        assert_errors!(diagnostic_reporter, [&expected]);
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

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(diagnostic_reporter, Vec::<Diagnostic>::new());
    }
}
