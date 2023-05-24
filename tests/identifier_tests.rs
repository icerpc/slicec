// Copyright (c) ZeroC, Inc.

mod test_helpers;

use crate::test_helpers::*;
use slicec::grammar::{CustomType, Exception, Interface, Struct};

#[test]
fn escaped_keywords() {
    // Arrange
    let slice = r#"
        module \module
        interface \interface {}
        exception \exception {}
        struct \struct {}
        custom \custom
    "#;

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("module::interface").is_ok());
    assert!(ast.find_element::<Exception>("module::exception").is_ok());
    assert!(ast.find_element::<Struct>("module::struct").is_ok());
    assert!(ast.find_element::<CustomType>("module::custom").is_ok());
}

#[test]
fn escaped_identifiers() {
    // Arrange
    let slice = r#"
        module \MyModule
        interface \MyInterface {}
        exception \MyException {}
        struct \MyStruct {}
        custom \MyCustom
    "#;

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("MyModule::MyInterface").is_ok());
    assert!(ast.find_element::<Exception>("MyModule::MyException").is_ok());
    assert!(ast.find_element::<Struct>("MyModule::MyStruct").is_ok());
    assert!(ast.find_element::<CustomType>("MyModule::MyCustom").is_ok());
}

#[test]
fn escaped_scoped_identifiers_containing_keywords() {
    // Arrange
    let slice = r#"
    module Foo

    struct \module {}

    struct BarStruct {
        s: Foo::\module
    }
    "#;

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Struct>("Foo::module").is_ok());
}
