// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod module {

    use crate::helpers::parsing_helpers::parse_for_ast;
    use slice::grammar::*;
    use slice::parse_from_strings;
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
        let err = parse_from_strings(&[slice], None).err();

        // Assert
        // TODO: better error message once we replace the parser
        assert!(err.is_some());
    }
}
