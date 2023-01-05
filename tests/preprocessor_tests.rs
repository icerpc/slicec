// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_diagnostics};
use slice::command_line::SliceOptions;
use slice::compile_from_strings;
use slice::grammar::*;
use test_case::test_case;

#[test]
fn command_line_defined_symbols() {
    // Arrange
    let slice = "
        module Test;

        # if Foo
        interface I
        {
            op();
        }
        # endif
        ";

    let options = SliceOptions {
        definitions: vec!["Foo".to_string()],
        ..Default::default()
    };

    // Act
    let compilation_data = compile_from_strings(&[slice], Some(options)).unwrap();

    // Assert
    assert!(compilation_data.ast.find_element::<Operation>("Test::I::op").is_ok());
}

#[test]
fn undefined_preprocessor_directive_blocks_are_consumed() {
    // Arrange
    let slice = "
            #if Foo
            module Test;
            interface I {}
            #endif
        ";

    // Act
    let compilation_data = compile_from_strings(&[slice], None).unwrap();

    // Assert
    assert!(compilation_data.ast.find_element::<Interface>("Test::I").is_err());
    assert_errors!(compilation_data.diagnostic_reporter);
}

#[test]
fn preprocessor_consumes_comments() {
    // Arrange
    let slice = "// This is a comment";

    // Act
    let reporter = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(reporter);
}

#[test]
fn preprocessor_define_symbol() {
    // Arrange
    let slice = "
        #define Foo
        #if Foo
        // Foo is defined
        module Test;
        interface I {}
        #endif
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("Test::I").is_ok());
}

#[test]
fn preprocessor_undefine_symbol() {
    // Arrange
    let slice = "
        #define Foo
        #undef Foo
        #if Foo
        // Foo is defined
        module Test;
        interface I {}
        #endif
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("Test::I").is_err());
}

#[test]
fn preprocessor_define_symbol_multiples_times() {
    // Arrange
    let slice = "
        #define Foo
        #define Foo
        #define Foo
        #if Foo
        // Foo is defined
        module Test;
        interface I {}
        #endif
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("Test::I").is_ok());
}
#[test_case("Foo", "I" ; "Foo is defined")]
#[test_case("Bar", "J" ; "Bar is defined")]
#[test_case("Baz", "K" ; "Baz is defined")]
#[test_case("Fizz", "X" ; "Fizz is defined")]
fn preprocessor_conditional_compilation(define: &str, interface: &str) {
    // Arrange
    let slice = format!(
        "
        #define {define}
        #if Foo

        // Foo is defined
        module Test;
        interface I {{}}

        # elif Bar

        // Bar is defined
        module Test;
        interface J {{}}

        # elif Baz

        // Baz is defined
        module Test;
        interface K {{}}

        # else

        // Fizz is defined
        module Test;
        interface X {{}}

        #endif
    "
    );

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast
        .find_element::<Interface>(format!("Test::{interface}").as_str())
        .is_ok());
}

#[test]
fn preprocessor_not_expressions() {
    // Arrange
    let slice = "
        #define Foo
        #if !Foo
        module Test;
        interface I {}
        #endif
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("Test::I").is_err());
}

#[test]
fn preprocessor_and_expressions() {
    // Arrange
    let slice = "
        #define Foo
        #define Bar
        #if Foo && Bar
        module Test;
        interface I {}
        #endif
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("Test::I").is_ok());
}

#[test]
fn preprocessor_or_expressions() {
    // Arrange
    let slice = "
        #define Foo
        #if Foo || Bar
        module Test;
        interface I {}
        #endif
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("Test::I").is_ok());
}

#[test]
fn preprocessor_grouped_expressions() {
    // Arrange
    let slice = "
        #define Foo
        #if (Foo || Bar) && (Fizz || Buzz)
        module Test;
        interface I {}
        #endif
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("Test::I").is_err());
}

#[test]
fn preprocessor_nested_expressions() {
    // Arrange
    let slice = "
        #define Bar
        #define Baz
        #if (Foo && (Bar || Baz))
        module Test;
        interface I {}
        #endif
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("Test::I").is_err());
}
