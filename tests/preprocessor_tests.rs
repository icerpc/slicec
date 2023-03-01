// Copyright (c) ZeroC, Inc.

pub mod helpers;

use crate::helpers::parsing_helpers::*;
use slice::command_line::SliceOptions;
use slice::compile_from_strings;
use slice::diagnostics::{Error, ErrorKind};
use slice::grammar::*;
use test_case::test_case;

#[test]
fn command_line_defined_symbols() {
    // Arrange
    let slice = "
        module Test

        # if Foo
        interface I {
            op()
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
            module Test
            interface I {}
            #endif
        ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("Test::I").is_err());
}

#[test]
fn preprocessor_consumes_comments() {
    // Arrange
    let slice = "// This is a comment";

    // Act/Assert
    assert_parses(slice);
}

#[test]
fn preprocessor_define_symbol() {
    // Arrange
    let slice = "
        #define Foo
        #if Foo
        // Foo is defined
        module Test
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
        module Test
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
        module Test
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
        module Test
        interface I {{}}

        # elif Bar

        // Bar is defined
        module Test
        interface J {{}}

        # elif Baz

        // Baz is defined
        module Test
        interface K {{}}

        # else

        // Fizz is defined
        module Test
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
        module Test
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
        module Test
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
        module Test
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
        module Test
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
        module Test
        interface I {}
        #endif
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("Test::I").is_err());
}

#[test_case(
    "
    #define Foo
    #if Foo
    #endif
    "
    ; "conditional"
)]
#[test_case(
    "
    #define Bar
    #if Foo
    #elif Bar
    #endif
    "
    ; "conditional with elif"
)]
#[test_case(
    "
    #if Foo
    #elif Bar
    #else
    #endif
    "
    ; "conditional with elif and else"
)]
fn preprocessor_conditionals_can_contain_empty_source_blocks(slice: &str) {
    assert_parses(slice);
}

#[test]
fn preprocessor_nested_conditional_blocks() {
    let slice = "
        #if !Foo
            module NotFooModule {}
            #if !Bar
                module NotBarModule {}
            #endif
        #else
            module ElseModule {}
        #endif
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Module>("NotFooModule").is_ok());
    assert!(ast.find_element::<Module>("NotBarModule").is_ok());
    assert!(ast.find_element::<Module>("ElseModule").is_err());
}

#[test]
fn preprocessor_ignores_comments() {
    // Arrange
    // If Bar is defined, then the comment was not ignored
    let slice = "
        #define Foo // define Bar
        #if Bar // This is a comment
        module Test
        interface I {}
        #endif // This is another comment
        // Hello
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Interface>("Test::I").is_err());
}

#[test]
fn preprocessor_single_backslash_suggestion() {
    // Arrange
    let slice = "
        # define bar / foo
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::Syntax {
        message: "unknown symbol '/', try using '//' instead".to_owned(),
    });
    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn preprocessor_recovers_at_end_of_line() {
    // Arrange
    let slice = "
        #define Foo Bar     // Error: can't define two things in one directive.

        #if Foo
            module Foo {}
        #elif (Bar          // Error: Missing a closing parenthesis.
            module Bar {}
        #endif
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = [
        Error::new(ErrorKind::Syntax {
            message: "expected one of directive_end, but found 'Identifier(\"Bar\")'".to_owned(),
        }),
        Error::new(ErrorKind::Syntax {
            message: r#"expected one of "&&", ")", or "||", but found 'DirectiveEnd'"#.to_owned(),
        }),
    ];
    check_diagnostics(diagnostics, expected);
}
