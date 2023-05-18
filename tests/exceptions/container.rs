// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};
use slicec::grammar::*;

/// Verifies that exceptions can contain fields.
#[test]
fn can_contain_fields() {
    // Arrange
    let slice = "
        module Test

        exception E {
            i: int32
            s: string
            b: bool
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let fields = ast.find_element::<Exception>("Test::E").unwrap().fields();

    assert_eq!(fields.len(), 3);
    assert!(matches!(fields[0].identifier(), "i"));
    assert!(matches!(fields[1].identifier(), "s"));
    assert!(matches!(fields[2].identifier(), "b"));
    assert!(matches!(
        fields[0].data_type.concrete_type(),
        Types::Primitive(Primitive::Int32),
    ));
    assert!(matches!(
        fields[1].data_type.concrete_type(),
        Types::Primitive(Primitive::String),
    ));
    assert!(matches!(
        fields[2].data_type.concrete_type(),
        Types::Primitive(Primitive::Bool),
    ));
}

/// Verifies that exceptions can be empty
#[test]
fn can_be_empty() {
    // Arrange
    let slice = "
        module Test

        exception E {}
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let fields = ast.find_element::<Exception>("Test::E").unwrap().fields();
    assert_eq!(fields.len(), 0);
}

#[test]
fn cannot_redefine_fields() {
    // Arrange
    let slice = "
        encoding = Slice1
        module Test

        exception E {
            a: int32
            a: string
        }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::Redefinition {
        identifier: "a".to_owned(),
    })
    .add_note("'a' was previously defined here", None);

    check_diagnostics(diagnostics, [expected]);
}
