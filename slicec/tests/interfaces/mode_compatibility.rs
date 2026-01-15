// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};
use slicec::grammar::CompilationMode;

#[test]
fn parameters_must_be_allowed_within_compilation_mode() {
    // Arrange
    let slice1 = "
        mode = Slice1
        module Test

        class C {}
    ";
    let slice2 = "
        mode = Slice2
        module Test

        interface I {
            op(c: C)
        }
    ";

    // Act
    let diagnostics = parse_multiple_for_diagnostics(&[slice1, slice2]);

    // Assert
    let expected = Diagnostic::new(Error::UnsupportedType {
        kind: "C".to_owned(),
        mode: CompilationMode::Slice2,
    });

    check_diagnostics(diagnostics, [expected]);
}
