// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

use crate::helpers::parsing_helpers::parse_for_diagnostics;
use slice::diagnostics::{Error, ErrorKind};
use slice::slice_file::Span;

#[test]
fn parse_empty_string() {
    // Arrange
    let slice = "";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(diagnostics);
}

#[test]
fn parse_string_containing_only_whitespace() {
    // Arrange
    let slice = " ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(diagnostics);
}

#[test]
fn parse_ideographic_space() {
    // Arrange
    // This is a special whitespace character U+3000 that is invisible.
    let slice = "　";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(diagnostics);
}

#[test]
fn string_literals_cannot_contain_newlines() {
    // Arrange
    let slice = r#"
        [foo("attribute
        test")]
        module Test;
    "#;

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let span = Span::new((2, 22).into(), (2, 32).into(), "string-0");
    let expected = Error::new(ErrorKind::Syntax {
        message: "unterminated string literal".to_owned(),
    })
    .set_span(&span);
    assert_errors!(diagnostics, [&expected]);
}
