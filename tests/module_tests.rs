// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod module {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_diagnostics};
    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::*;
    use slice::compile_from_strings;
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
        let err = compile_from_strings(&[slice], None).err();

        // Assert
        // TODO: better error message once we replace the parser
        assert!(err.is_some());
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
            Error::new(ErrorKind::FileScopedModuleCannotContainSubModules("A".to_owned()), None),
            Error::new(ErrorKind::FileScopedModuleCannotContainSubModules("A".to_owned()), None),
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
        let expected = vec![Error::new(
            ErrorKind::FileScopedModuleCannotContainSubModules("D".to_owned()),
            None,
        )];
        assert_errors!(errors, expected);
    }
}
