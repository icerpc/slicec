// Copyright (c) ZeroC, Inc.

mod encoding;
mod inheritance;
mod operations;

use crate::test_helpers::*;
use slice::diagnostics::{Error, ErrorKind};
use slice::grammar::*;

#[test]
fn can_have_no_operations() {
    // Arrange
    let slice = "
        module Test

        interface I {}
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let interface_def = ast.find_element::<Interface>("Test::I").unwrap();
    assert_eq!(interface_def.identifier(), "I");
    assert_eq!(interface_def.operations().len(), 0);
}

#[test]
fn can_have_self_referencing_operations() {
    // Arrange
    let slice = "
        module Test

        interface I {
            myOp() -> I
        }
    ";

    // Act/Assert
    assert_parses(slice);
}

#[test]
fn can_have_one_operation() {
    // Arrange
    let slice = "
        module Test

        interface I {
            op1()
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let interface_def = ast.find_element::<Interface>("Test::I").unwrap();
    assert_eq!(interface_def.operations().len(), 1);
}

#[test]
fn can_have_multiple_operation() {
    // Arrange
    let slice = "
        module Test

        interface I {
            op1()
            op2()
            op3()
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let interface_def = ast.find_element::<Interface>("Test::I").unwrap();
    assert_eq!(interface_def.operations().len(), 3);
}

#[test]
fn cannot_redefine_operations() {
    // Arrange
    let slice = "
        encoding = 1
        module Test

        interface I {
            op()
            op()
        }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::Redefinition {
        identifier: "op".to_owned(),
    })
    .add_note("'op' was previously defined here", None);

    check_diagnostics(diagnostics, [expected]);
}
