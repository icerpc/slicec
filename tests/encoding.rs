// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

use crate::helpers::parsing_helpers::parse_for_errors;
use test_case::test_case;

/// Verifies that the supported encodings compile
#[test_case("1")]
#[test_case("2")]
fn valid_encodings(value: &str) {
    // Arrange
    let slice = &format!(
        "
        encoding = {value};
        ",
        value = value,
    );

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors!(error_reporter);
}

#[test]
#[should_panic] // TODO: Fix parse_for_errors to not panic
fn invalid_encodings_fail() {
    // Arrange
    let slice = "
        encoding = 3;
        ";
    let expected_errors: &[&str] = &[];

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors!(error_reporter, expected_errors);
}
