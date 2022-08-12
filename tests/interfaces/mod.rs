// Copyright (c) ZeroC, Inc. All rights reserved.

mod encoding;
mod inheritance;
mod operations;

use crate::assert_errors_new;
use crate::helpers::parsing_helpers::*;
use slice::diagnostics::{DiagnosticKind, LogicKind};
use slice::grammar::*;
use slice::parse_from_string;

#[test]
fn can_have_no_operations() {
    // Arrange
    let slice = "
        module Test;
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
        module Test;
        interface I {
            myOp() -> I;
        }
    ";

    // Act
    let result = parse_from_string(slice);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn can_have_one_operation() {
    // Arrange
    let slice = "
        module Test;
        interface I
        {
            op1();
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
        module Test;
        interface I
        {
            op1();
            op2();
            op3();
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
        encoding = 1;
        module Test;
        interface I
        {
            op();
            op();
        }
    ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected: [DiagnosticKind; 2] = [
        LogicKind::Redefinition("op".to_owned()).into(),
        DiagnosticKind::new_note("`op` was previously defined here".to_owned()),
    ];
    assert_errors_new!(diagnostic_reporter, expected);
}
