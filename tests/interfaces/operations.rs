// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::*;
use slice::grammar::*;

#[test]
fn can_have_no_parameters() {
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
fn can_have_no_return_type() {
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
fn can_have_parameters() {
    let slice = "
        module Test;
        interface I
        {
            op(a: int32, b: string, c: varuint62);
        }
    ";

    let ast = parse_for_ast(slice);
    let interface_ptr = ast.find_typed_type::<Interface>("Test::I").unwrap();
    let interface_def = interface_ptr.borrow();
    let operation = interface_def.operations()[0];
    let parameters = operation.parameters();

    assert_eq!(parameters.len(), 3);
    assert_eq!(parameters[0].identifier(), "a");
    assert_eq!(parameters[1].identifier(), "b");
    assert_eq!(parameters[2].identifier(), "c");

    assert!(matches!(
        parameters[0].data_type.concrete_type(),
        Types::Primitive(Primitive::Int32)
    ));
    assert!(matches!(
        parameters[1].data_type.concrete_type(),
        Types::Primitive(Primitive::String)
    ));
    assert!(matches!(
        parameters[2].data_type.concrete_type(),
        Types::Primitive(Primitive::VarUInt62)
    ));
}

#[test]
fn can_have_return_value() {
    let slice = "
        module Test;
        interface I
        {
            op() -> string;
        }
    ";

    let ast = parse_for_ast(slice);
    let interface_ptr = ast.find_typed_type::<Interface>("Test::I").unwrap();
    let interface_def = interface_ptr.borrow();
    let operation = interface_def.operations()[0];
    let returns = operation.return_members();

    assert_eq!(returns.len(), 1);

    assert_eq!(returns[0].identifier(), "returnValue");

    assert!(matches!(
        returns[0].data_type.concrete_type(),
        Types::Primitive(Primitive::String)
    ));
}

#[test]
fn can_have_return_tuple() {
    let slice = "
        module Test;
        interface I
        {
            op() -> (r1: string, r2: bool);
        }
    ";

    let ast = parse_for_ast(slice);
    let interface_ptr = ast.find_typed_type::<Interface>("Test::I").unwrap();
    let interface_def = interface_ptr.borrow();
    let operation = interface_def.operations()[0];
    let returns = operation.return_members();

    assert_eq!(returns.len(), 2);

    assert_eq!(returns[0].identifier(), "r1");
    assert_eq!(returns[1].identifier(), "r2");

    assert!(matches!(
        returns[0].data_type.concrete_type(),
        Types::Primitive(Primitive::String)
    ));
    assert!(matches!(
        returns[1].data_type.concrete_type(),
        Types::Primitive(Primitive::Bool)
    ));
}

#[test]
#[ignore] // should fail
fn return_tuple_can_not_be_empty() {
    let slice = "
        module Test;
        interface I
        {
            op() -> ();
        }
    ";

    let error_reporter = parse_for_errors(slice);

    error_reporter.assert_errors(&["Expected at least two return value"]);
}

#[test]
#[ignore] // should fail
fn return_tuple_can_not_have_one_element() {
    let slice = "
        module Test;
        interface I
        {
            op() -> (a: int32);
        }
    ";

    let error_reporter = parse_for_errors(slice);

    error_reporter.assert_errors(&["Tuple returns require at least two values"]);
}

mod streams {
    use crate::helpers::parsing_helpers::*;
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
        error_reporter.assert_errors(&["only the last parameter in an operation can be streamed"]);
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
        error_reporter.assert_errors(&["only the last parameter in an operation can be streamed"]);
    }
}
