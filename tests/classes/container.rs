// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slice::diagnostics::{Error, ErrorKind};
use slice::grammar::*;
use test_case::test_case;

/// Verifies that classes can contain fields.
#[test]
fn can_contain_fields() {
    // Arrange
    let slice = "
        encoding = 1
        module Test
        class C {
            i: int32
            s: string
            b: bool
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let fields = ast.find_element::<Class>("Test::C").unwrap().fields();

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

#[test_case(
    "
        class C {
            c: C
        }
    "; "single class circular reference"
)]
#[test_case(
    "
        class C1 {
            c2: C2
        }
        class C2 {
            c1: C1
        }
    "; "multi class circular reference"
)]
fn cycles_are_allowed(cycle_string: &str) {
    // Arrange
    let slice = format!(
        "
            encoding = 1
            module Test
            {cycle_string}
        "
    );

    // Act/Assert
    assert_parses(slice);
}

/// Verifies that classes can be empty
#[test]
fn can_be_empty() {
    // Arrange
    let slice = "
        encoding = 1
        module Test
        class C {}
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let fields = ast.find_element::<Class>("Test::C").unwrap().fields();
    assert_eq!(fields.len(), 0);
}

#[test]
fn cannot_redefine_fields() {
    // Arrange
    let slice = "
        encoding = 1
        module Test
        class C {
            a: int32
            a: string
        }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::Redefinition {
        identifier: "a".to_string(),
    })
    .add_note("'a' was previously defined here", None);

    check_diagnostics(diagnostics, [expected]);
}
