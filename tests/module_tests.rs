// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod module {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::*;
    use slice::diagnostics::{Error, ErrorKind, Note};
    use slice::grammar::*;
    use slice::parse_from_strings;

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
        let expected = Error::new_with_notes(ErrorKind::Redefinition("Bar".to_owned()), None, vec![Note::new(
            "`Bar` was previously defined here",
            None,
        )]);
        assert_errors!(diagnostic_reporter, [&expected]);
    }
}
