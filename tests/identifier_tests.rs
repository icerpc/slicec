// Copyright (c) ZeroC, Inc.

mod test_helpers;

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};
use slicec::grammar::{CustomType, Interface, Struct};

#[test]
fn escaped_keywords() {
    // Arrange
    let slice = r#"
        module \module
        interface \interface {}
        struct \struct {}
        custom \custom
    "#;

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("module::interface").is_ok());
    assert!(ast.find_element::<Struct>("module::struct").is_ok());
    assert!(ast.find_element::<CustomType>("module::custom").is_ok());
}

#[test]
fn escaped_identifiers() {
    // Arrange
    let slice = r#"
        module \MyModule
        interface \MyInterface {}
        struct \MyStruct {}
        custom \MyCustom
    "#;

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("MyModule::MyInterface").is_ok());
    assert!(ast.find_element::<Struct>("MyModule::MyStruct").is_ok());
    assert!(ast.find_element::<CustomType>("MyModule::MyCustom").is_ok());
}

#[test]
fn must_start_with_a_letter() {
    // Arrange
    let slice = "module _foo";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::Syntax {
        message: "unknown symbol '_'".to_owned(),
    });
    check_diagnostics(diagnostics, [expected]);
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

#[test]
fn must_be_ascii_alphanumeric_characters() {
    // Arrange
    let slice = "module 𒅋";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::Syntax {
        message: "unknown symbol '𒅋'".to_owned(),
    });
    check_diagnostics(diagnostics, [expected]);
}
