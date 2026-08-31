// Copyright (c) ZeroC, Inc.

mod test_helpers;

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};
use slicec::grammar::{CustomType, Field, Interface, Module, NamedSymbol, Primitive, Struct, Types};

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
    assert!(ast.find_symbol_by_id::<Interface>("module::interface").is_ok());
    assert!(ast.find_symbol_by_id::<Struct>("module::struct").is_ok());
    assert!(ast.find_symbol_by_id::<CustomType>("module::custom").is_ok());
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
    assert!(ast.find_symbol_by_id::<Interface>("MyModule::MyInterface").is_ok());
    assert!(ast.find_symbol_by_id::<Struct>("MyModule::MyStruct").is_ok());
    assert!(ast.find_symbol_by_id::<CustomType>("MyModule::MyCustom").is_ok());
}

#[test]
fn module_named_after_primitive_keyword_is_allowed() {
    // Arrange
    let slice = r#"
        module \int32

        struct Foo {
            x: int32
        }
    "#;

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    // The module parses correctly, and has an identifier of "int32".
    assert!(ast.find_symbol_by_id::<Module>("int32").is_ok());

    // The field in `Foo` still resolves to the `int32` primitive type, not the module.
    let field = ast.find_symbol_by_id::<Field>("int32::Foo::x").unwrap();
    assert!(matches!(
        field.data_type.concrete_type(),
        Types::Primitive(Primitive::Int32)
    ));
}

#[test]
fn top_level_element_named_after_primitive_keyword_is_allowed() {
    // Arrange
    let slice = r#"
        module Test

        struct \string {}

        struct Foo {
            a: string
            b: \string
        }
    "#;

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    // The struct parses correctly, and has an identifier of "string".
    let string_struct = ast.find_symbol_by_id::<Struct>("Test::string").unwrap();
    assert_eq!(string_struct.identifier(), "string");

    // The fields in `Foo` correctly resolve to the primitive type, and the struct, depending on escaping.
    let field_a = ast.find_symbol_by_id::<Field>("Test::Foo::a").unwrap();
    assert!(matches!(
        field_a.data_type.concrete_type(),
        Types::Primitive(Primitive::String)
    ));
    let field_b = ast.find_symbol_by_id::<Field>("Test::Foo::b").unwrap();
    assert!(matches!(field_b.data_type.concrete_type(), Types::Struct(_)));
}

#[test]
fn keyword_named_types_without_a_module_do_not_panic() {
    // Arrange
    let slice = r#"
        struct \int32 {}

        struct Foo {
            f: int32
        }
    "#;

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::from_error(Error::Syntax {
        message: "module declaration is required".to_owned(),
    });
    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn elements_in_modules_named_after_primitive_keywords_are_referenceable() {
    // Arrange
    let slice1 = r#"
        module Hello::\int32

        struct Foo {}
    "#;
    let slice2 = r#"
        module Test

        struct S {
            a: Hello::\int32::Foo
        }
    "#;

    // Act
    let ast = parse_multiple_for_ast(&[slice1, slice2]);

    // Assert
    let field = ast.find_symbol_by_id::<Field>("Test::S::a").unwrap();
    let Types::Struct(struct_def) = field.data_type.concrete_type() else {
        panic!("field type was not a struct");
    };
    assert_eq!(struct_def.module_scoped_identifier(), "Hello::int32::Foo");
}

#[test]
fn must_start_with_a_letter() {
    // Arrange
    let slice = "module _foo";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::from_error(Error::Syntax {
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
    assert!(ast.find_symbol_by_id::<Struct>("Foo::module").is_ok());
}

#[test]
fn must_be_ascii_alphanumeric_characters() {
    // Arrange
    let slice = "module 𒅋";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::from_error(Error::Syntax {
        message: "unknown symbol '𒅋'".to_owned(),
    });
    check_diagnostics(diagnostics, [expected]);
}
