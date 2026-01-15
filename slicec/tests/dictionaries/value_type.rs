// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};

#[test]
fn invalid_dictionary_values_produce_error() {
    // Arrange
    let slice = "
    module Foo
    struct Bar {
         i: Dictionary<int32, Dictionary<float32, bool>>
         j: Dictionary<int32, Sequence<Dictionary<float64, bool>>>
    }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = [
        Diagnostic::new(Error::KeyTypeNotSupported {
            kind: "float32".to_owned(),
        }),
        Diagnostic::new(Error::KeyTypeNotSupported {
            kind: "float64".to_owned(),
        }),
    ];
    check_diagnostics(diagnostics, expected);
}
