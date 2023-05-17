// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slice::diagnostics::{Diagnostic, Error};

#[test]
fn invalid_dictionary_values_produce_error() {
    // Arrange
    let slice = "
    module Foo
    struct Bar {
         i: dictionary<int32, dictionary<float32, bool>>
         j: dictionary<int32, sequence<dictionary<float64, bool>>>
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
