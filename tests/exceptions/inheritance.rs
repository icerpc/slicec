// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::*;
use slice::grammar::*;

#[test]
fn supports_single_inheritance() {
    let slice = "
        encoding = 1;
        module Test;
        exception E1 {}
        exception E2 : E1 {}";

    let ast = parse_for_ast(slice);

    let e2_ptr = ast.find_typed_type::<Exception>("Test::E2").unwrap();
    let e2 = e2_ptr.borrow();
    assert_eq!(
        e2.base_exception().unwrap().module_scoped_identifier(),
        "Test::E1"
    );
}

#[test]
fn does_not_support_multiple_inheritance() {
    let slice = "
        encoding = 1;
        module Test;
        exception E1 {}
        exception E2 {}
        exception E3 : E1, E2 {}
        ";

    let error_reporter = parse_for_errors(slice);

    assert_errors!(error_reporter, [
        "exceptions can only inherit from a single base exception"
    ]);
}

#[test]
fn must_inherit_from_exception() {
    let slice = "
    encoding = 1;
    module Test;
    class C {}
    exception E : C {}
    ";

    let error_reporter = parse_for_errors(slice);

    assert_errors!(error_reporter, &[
        "The Entity 'C' is not a valid type for this definition.",
    ]);
}

#[test]
fn inherits_correct_data_members() {
    let slice = "
    encoding = 1;
    module Test;
    exception A
    {
        a: int32
    }
    exception B : A
    {
        b: string
    }
    exception C : B
    {
        c: float64
    }
    ";

    let ast = parse_for_ast(slice);
    let exception_a_ptr = ast.find_typed_type::<Exception>("Test::A").unwrap();
    let exception_b_ptr = ast.find_typed_type::<Exception>("Test::B").unwrap();
    let exception_c_ptr = ast.find_typed_type::<Exception>("Test::C").unwrap();

    let exception_a_def = exception_a_ptr.borrow();
    let exception_b_def = exception_b_ptr.borrow();
    let exception_c_def = exception_c_ptr.borrow();

    assert_eq!(exception_a_def.members().len(), 1);
    assert_eq!(exception_a_def.all_members().len(), 1);
    assert_eq!(exception_a_def.all_members()[0].identifier(), "a");

    assert_eq!(exception_b_def.members().len(), 1);
    assert_eq!(exception_b_def.all_members().len(), 2);
    assert_eq!(exception_b_def.all_members()[0].identifier(), "a");
    assert_eq!(exception_b_def.all_members()[1].identifier(), "b");

    assert_eq!(exception_c_def.members().len(), 1);
    assert_eq!(exception_c_def.all_members().len(), 3);
    assert_eq!(exception_c_def.all_members()[0].identifier(), "a");
    assert_eq!(exception_c_def.all_members()[1].identifier(), "b");
    assert_eq!(exception_c_def.all_members()[2].identifier(), "c");
}
