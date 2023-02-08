// Copyright (c) ZeroC, Inc.

pub mod helpers;

use helpers::parsing_helpers::parse_for_ast;
use slice::grammar::{Enum, Exception, Interface, Struct};

#[test]
fn escaped_keywords() {
    // Arrange
    let slice = r#"
        module \module;
        interface \interface {}
        exception \exception {}
        struct \struct {}
        enum \enum {}
    "#;

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("module::interface").is_ok());
    assert!(ast.find_element::<Exception>("module::exception").is_ok());
    assert!(ast.find_element::<Struct>("module::struct").is_ok());
    assert!(ast.find_element::<Enum>("module::enum").is_ok());
}

#[test]
fn escaped_identifiers() {
    // Arrange
    let slice = r#"
        module \MyModule;
        interface \MyInterface {}
        exception \MyException {}
        struct \MyStruct {}
        enum \MyEnum {}
    "#;

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("MyModule::MyInterface").is_ok());
    assert!(ast.find_element::<Exception>("MyModule::MyException").is_ok());
    assert!(ast.find_element::<Struct>("MyModule::MyStruct").is_ok());
    assert!(ast.find_element::<Enum>("MyModule::MyEnum").is_ok());
}

#[test]
fn escaped_scoped_identifiers_containing_keywords() {
    // Arrange
    let slice = r#"
    module Foo
    {
        struct \module {}
    }

    module Bar
    {
        struct BarStruct
        {
            s: Foo::\module
        }
    }
    "#;

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Struct>("Foo::module").is_ok());
}
