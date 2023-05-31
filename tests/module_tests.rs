// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod module {
    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use slicec::grammar::*;

    #[test]
    fn can_be_defined() {
        // Arrange
        let slice = "module Test";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        assert!(ast.find_element::<Module>("Test").is_ok());
    }

    #[test]
    fn can_use_nested_syntax() {
        // Arrange
        let slice = "
            module A::B::C::D
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
            custom C
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::Syntax {
            message: "module declaration is required".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn multiple_files_can_use_the_same_module() {
        // Arrange
        let slice1 = "
            module Foo
            struct Test1 {}
        ";
        let slice2 = "
            module Foo
            struct Test2 {}
        ";

        // Act
        let ast = parse_multiple_for_ast(&[slice1, slice2]);

        // Assert
        assert!(ast.find_element::<Struct>("Foo::Test1").is_ok());
        assert!(ast.find_element::<Struct>("Foo::Test2").is_ok());
    }

    #[test]
    fn cross_module_redefinitions_are_disallowed() {
        // Arrange
        let slice1 = "
            module Foo
            struct Bar {}
        ";
        let slice2 = "
            module Foo
            exception Bar {}
        ";

        // Act
        let diagnostics = parse_multiple_for_diagnostics(&[slice1, slice2]);

        // Assert
        let expected = Diagnostic::new(Error::Redefinition {
            identifier: "Bar".to_owned(),
        })
        .add_note("'Bar' was previously defined here", None);

        check_diagnostics(diagnostics, [expected]);
    }
}
