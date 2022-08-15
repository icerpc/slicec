// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors_new;
use slice::diagnostics::{DiagnosticKind, LogicErrorKind};
use slice::grammar::Encoding;
use slice::parse_from_strings;

#[test]
fn operation_members_are_compatible_with_encoding() {
    // Arrange
    let slice1 = "
        encoding = 1;
        module Test;
        class C {}
    ";
    let slice2 = "
        encoding = 2;
        module Test;
        interface I {
            op(c: C);
        }
    ";

    // Act
    let result = parse_from_strings(&[slice1, slice2]).err().unwrap();

    // Assert
    let expected = [
        LogicErrorKind::UnsupportedType("C".to_owned(), Encoding::Slice2).into(),
        DiagnosticKind::new_note("file encoding was set to Slice2 here:".to_owned()),
    ];
    assert_errors_new!(result.diagnostic_reporter, expected);
}
