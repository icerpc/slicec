// Copyright (c) ZeroC, Inc. All rights reserved.

mod encoding;
mod inheritance;
mod operations;

use crate::assert_errors_new;
use crate::helpers::parsing_helpers::*;
use slice::errors::{ErrorKind, RuleKind};
use slice::grammar::*;
use slice::parse_from_string;

#[test]
fn can_have_no_operations() {
    let slice = "
        module Test;
        interface I {}
    ";

    let ast = parse_for_ast(slice);

    let interface_def = ast.find_element::<Interface>("Test::I").unwrap();
    assert_eq!(interface_def.identifier(), "I");
    assert_eq!(interface_def.operations().len(), 0);
}

#[test]
fn can_have_self_referencing_operations() {
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
    let slice = "
        module Test;
        interface I
        {
            op1();
        }
    ";

    let ast = parse_for_ast(slice);

    let interface_def = ast.find_element::<Interface>("Test::I").unwrap();
    assert_eq!(interface_def.operations().len(), 1);
}

#[test]
fn can_have_multiple_operation() {
    let slice = "
        module Test;
        interface I
        {
            op1();
            op2();
            op3();
        }
    ";

    let ast = parse_for_ast(slice);

    let interface_def = ast.find_element::<Interface>("Test::I").unwrap();
    assert_eq!(interface_def.operations().len(), 3);
}

#[test]
fn cannot_redefine_operations() {
    let slice = "
        encoding = 1;
        module Test;
        interface I
        {
            op();
            op();
        }
    ";
    let expected: [ErrorKind; 2] = [
        RuleKind::Redefinition("op".to_owned()).into(),
        ErrorKind::new("`op` was previously defined here".to_owned()),
    ];

    let error_reporter = parse_for_errors(slice);

    assert_errors_new!(error_reporter, expected);
}
