// Copyright (c) ZeroC, Inc.

pub mod test_helpers;

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};
use slicec::grammar::*;
use slicec::slice_options::SliceOptions;
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
        defined_symbols: vec!["Foo".to_string()],
        ..Default::default()
    };

    // Act
    let compilation_state = parse(slice, Some(&options));

    // Assert
    assert!(compilation_state.ast.find_element::<Operation>("Test::I::op").is_ok());
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

#[test_case("6foo"; "numeric")]
#[test_case("_foo"; "underscore")]
fn identifiers_must_start_with_a_letter(identifier: &str) {
    // Arrange
    let slice = format!("#define {identifier}");

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::Syntax {
        message: format!("unknown symbol '{}'", identifier.chars().next().unwrap()),
    });
    check_diagnostics(diagnostics, [expected]);
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
        module Test

        #if !Foo
            struct NotFooStruct {}
            #if !Bar
                struct NotBarStruct {}
            #endif
        #else
            struct ElseStruct {}
        #endif
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert!(ast.find_element::<Struct>("Test::NotFooStruct").is_ok());
    assert!(ast.find_element::<Struct>("Test::NotBarStruct").is_ok());
    assert!(ast.find_element::<Struct>("Test::ElseStruct").is_err());
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
    let expected = Diagnostic::new(Error::Syntax {
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
            module bool {}  // Doesn't emit an error because parsing stops after preprocessing.
        #elif (Bar          // Error: Missing a closing parenthesis.
            module bool {}  // Doesn't emit an error because parsing stops after preprocessing.
        #endif
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = [
        Diagnostic::new(Error::Syntax {
            message: "expected directive_end, but found 'Identifier(\"Bar\")'".to_owned(),
        }),
        Diagnostic::new(Error::Syntax {
            message: r#"expected one of "&&", "||", or ")", but found 'DirectiveEnd'"#.to_owned(),
        }),
    ];
    check_diagnostics(diagnostics, expected);
}
