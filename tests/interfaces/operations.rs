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
