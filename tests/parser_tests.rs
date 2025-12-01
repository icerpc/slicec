// Copyright (c) ZeroC, Inc.

mod test_helpers;

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};
use slicec::grammar::{attributes, AttributeFunctions, Enumerator, Struct};
use slicec::slice_file::Span;

#[test]
fn parse_empty_string() {
    // Arrange
    let slice = "";

    // Act/Assert
    assert_parses(slice);
}

#[test]
fn parse_string_containing_only_whitespace() {
    // Arrange
    let slice = " ";

    // Act/Assert
    assert_parses(slice);
}

#[test]
fn parse_ideographic_space() {
    // Arrange
    // This is a special whitespace character U+3000 that is invisible.
    let slice = "　";

    // Act/Assert
    assert_parses(slice);
}

#[test]
fn string_literals_cannot_contain_newlines() {
    // Arrange
    let slice = r#"
        [foo("attribute
        test")]
        module Test
    "#;

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let span = Span::new((2, 14).into(), (2, 24).into(), "string-0");
    let expected = Diagnostic::new(Error::Syntax {
        message: "unterminated string literal".to_owned(),
    })
    .set_span(&span);

    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn string_literals_support_character_escaping() {
    // Arrange
    let slice = r#"
        module Test

        [deprecated("This is a backslash\"\\\"\n.")]
        struct Foo {}
    "#;

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let struct_def = ast.find_element::<Struct>("Test::Foo").unwrap();
    let deprecated = struct_def.find_attribute::<attributes::Deprecated>().unwrap();
    assert_eq!(deprecated.reason, Some("This is a backslash\"\\\"n.".to_owned()))
}

#[test]
fn integer_literals_can_contain_underscores() {
    // Arrange
    let slice = "
        module Test

        enum Foo : int32 {
            A = 17_000_000
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let enumerator = ast.find_element::<Enumerator>("Test::Foo::A").unwrap();
    assert_eq!(enumerator.value(), 17_000_000);
}

// Ensure a syntax error in one file doesn't affect how we parse other files; See: github.com/icerpc/slicec/issues/559.
#[test]
fn files_are_parsed_independently() {
    // Arrange
    let slice1 = "
        module Not-Valid
    ";
    let slice2 = "
        module Also-Bogus
    ";

    // Act
    let diagnostics = parse_multiple_for_diagnostics(&[slice1, slice2]);

    // Assert
    let expected_message = "expected one of 'doc comment', 'struct', 'exception', 'class', 'interface', 'enum', 'custom', 'typealias', 'compact', 'unchecked', '[', or '::', but found '-'";
    let expected = [
        Diagnostic::new(Error::Syntax {
            message: expected_message.to_owned(),
        }),
        Diagnostic::new(Error::Syntax {
            message: expected_message.to_owned(),
        }),
    ];
    check_diagnostics(diagnostics, expected);
}
