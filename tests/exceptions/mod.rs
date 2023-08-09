// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};

mod container;
mod inheritance;
mod mode_compatibility;
mod tags;

#[test]
fn cannot_be_used_as_types() {
    // Arrange
    let slice = "
        mode = Slice1
        module Test

        exception E {}
        compact struct S {
            e: E
        }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::TypeMismatch {
        expected: "type".to_owned(),
        actual: "exception".to_owned(),
        is_concrete: false,
    });
    check_diagnostics(diagnostics, [expected]);
}
