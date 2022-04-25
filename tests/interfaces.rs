// Copyright (c) ZeroC, Inc. All rights reserved.

use slice::ast::Ast;
use slice::error::ErrorReporter;
use slice::grammar::*;
use slice::parse_from_string;

fn parse_for_ast(slice: &str) -> Ast {
    let (ast, error_reporter) = parse_from_string(slice).ok().unwrap();
    assert!(!error_reporter.has_errors(true));
    ast
}

fn parse_for_errors(slice: &str) -> ErrorReporter {
    let (_, error_reporter) = parse_from_string(slice).ok().unwrap();
    error_reporter
}

#[test]
fn can_have_no_operations() {
    let slice = "
        module Test;
        interface I {}
    ";

    let ast = parse_for_ast(slice);
    let interface_ptr = ast.find_typed_type::<Interface>("Test::I").unwrap();
    let interface_def = interface_ptr.borrow();
    assert_eq!(interface_def.identifier(), "I");
    assert_eq!(interface_def.operations().len(), 0);
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
    let interface_ptr = ast.find_typed_type::<Interface>("Test::I").unwrap();
    let interface_def = interface_ptr.borrow();

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
    let interface_ptr = ast.find_typed_type::<Interface>("Test::I").unwrap();
    let interface_def = interface_ptr.borrow();

    assert_eq!(interface_def.operations().len(), 3);
}

mod operations {
    use super::*;
    #[test]
    fn operation_can_have_no_parameters() {
        let slice = "
            module Test;
            interface I
            {
                op();
            }
        ";

        let ast = parse_for_ast(slice);
        let interface_ptr = ast.find_typed_type::<Interface>("Test::I").unwrap();
        let interface_def = interface_ptr.borrow();

        let operation = interface_def.operations()[0];

        assert!(operation.parameters().is_empty());
    }

    #[test]
    fn operation_can_have_no_response() {
        let slice = "
            module Test;
            interface I
            {
                op(a: int32);
            }
        ";

        let ast = parse_for_ast(slice);
        let interface_ptr = ast.find_typed_type::<Interface>("Test::I").unwrap();
        let interface_def = interface_ptr.borrow();

        let operation = interface_def.operations()[0];

        assert!(operation.return_members().is_empty());
    }

    #[test]
    fn operation_can_have_at_most_one_streamed_parameter() {
        let slice = "
            module Test;
            interface I
            {
                op(s: stream varuint62, s2: stream string);
            }
        ";

        let error_reporter = parse_for_errors(slice);
        error_reporter.assert_errors(&["can not be more than one streamed parameter"]);
    }

    #[test]
    fn stream_parameter_must_be_last() {
        let slice = "
            module Test;
            interface I
            {
                op(s: stream varuint62, i: int32);
            }
        ";

        let error_reporter = parse_for_errors(slice);
        error_reporter.assert_errors(&["can not be more than one streamed parameter"]);
    }
}
