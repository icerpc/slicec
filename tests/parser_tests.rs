// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::command_line::SliceOptions;
use slice::compile_from_strings;
use slice::grammar::Interface;

#[test]
fn parse_empty_file() {
    // Arrange
    let slice = "";

    // Act
    let compilation_data = compile_from_strings(&[slice], Some(SliceOptions::default())).unwrap();

    // Assert
    assert!(!compilation_data.diagnostic_reporter.has_errors());
}

#[test]
fn parse_empty_file_with_preprocessor_directive() {
    // Arrange
    let slice = "
    #if Foo
    module Test;
    interface I {}
    #endif
";

    // Act
    let compilation_data = compile_from_strings(&[slice], Some(SliceOptions::default())).unwrap();

    // Assert
    assert!(compilation_data.ast.find_element::<Interface>("Test::I").is_err());
    assert!(!compilation_data.diagnostic_reporter.has_errors());
}

#[test]
fn parse_empty_file_with_comment() {
    // Arrange
    let slice = "// This is a comment";

    // Act
    let compilation_data = compile_from_strings(&[slice], Some(SliceOptions::default())).unwrap();

    // Assert
    assert!(!compilation_data.diagnostic_reporter.has_errors());
}

#[test]
fn parse_empty_file_with_whitespace() {
    // Arrange
    let slice = " ";

    // Act
    let compilation_data = compile_from_strings(&[slice], Some(SliceOptions::default())).unwrap();

    // Assert
    assert!(!compilation_data.diagnostic_reporter.has_errors());
}

#[test]
fn parse_empty_file_with_ideographic_space() {
    // Arrange
    let slice = "　";

    // Act
    let compilation_data = compile_from_strings(&[slice], Some(SliceOptions::default())).unwrap();

    // Assert
    assert!(!compilation_data.diagnostic_reporter.has_errors());
}
