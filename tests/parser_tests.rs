// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::command_line::SliceOptions;
use slice::compile_from_strings;

#[test]
fn parse_empty_string() {
    // Arrange
    let slice = "";

    // Act
    let compilation_data = compile_from_strings(&[slice], Some(SliceOptions::default())).unwrap();

    // Assert
    assert!(!compilation_data.diagnostic_reporter.has_errors());
}

#[test]
fn parse_string_containing_whitespace() {
    // Arrange
    let slice = " ";

    // Act
    let compilation_data = compile_from_strings(&[slice], Some(SliceOptions::default())).unwrap();

    // Assert
    assert!(!compilation_data.diagnostic_reporter.has_errors());
}

#[test]
fn parse_ideographic_space() {
    // Arrange
    let slice = "　";

    // Act
    let compilation_data = compile_from_strings(&[slice], Some(SliceOptions::default())).unwrap();

    // Assert
    assert!(!compilation_data.diagnostic_reporter.has_errors());
}
