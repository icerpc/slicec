// Copyright (c) ZeroC, Inc.

use crate::assert_errors;
use crate::helpers::parsing_helpers::parse_for_diagnostics;
use slice::diagnostics::{Error, ErrorKind};

#[test]
fn invalid_dictionary_values_produce_error() {
    // Arrange
    let slice = "
    module Foo;
    struct Bar {
         i: dictionary<int32, dictionary<float32, bool>>,
         j: dictionary<int32, sequence<dictionary<float64, bool>>>,
    }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = vec![
        Error::new(ErrorKind::KeyTypeNotSupported {
            kind: "float32".to_owned(),
        }),
        Error::new(ErrorKind::KeyTypeNotSupported {
            kind: "float64".to_owned(),
        }),
    ];
    assert_errors!(diagnostics, expected);
}
