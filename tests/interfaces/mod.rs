// Copyright (c) ZeroC, Inc. All rights reserved.

mod operations;

use crate::helpers::parsing_helpers::*;
use slice::grammar::*;

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

#[test]
fn supports_single_inheritance() {
    let slice = "
        module Test;
        interface I {}
        interface J : I {}
    ";

    let ast = parse_for_ast(slice);
    let interface_i_ptr = ast.find_typed_type::<Interface>("Test::I").unwrap();
    let interface_j_ptr = ast.find_typed_type::<Interface>("Test::J").unwrap();
    let interface_i_def = interface_i_ptr.borrow();
    let interface_j_def = interface_j_ptr.borrow();
    let interface_j_bases = interface_j_def.base_interfaces();

    assert!(interface_i_def.base_interfaces().is_empty());
    assert_eq!(interface_j_bases.len(), 1);
    assert_eq!(
        interface_j_bases[0].module_scoped_identifier(),
        interface_i_def.module_scoped_identifier(),
    );
}

#[test]
fn supports_multiple_inheritance() {
    let slice = "
        module Test;
        interface I
        {
            op1();
        }
        interface J
        {
            op2();
        }
        interface K : I, J
        {
            op2();
        }
    ";

    let ast = parse_for_ast(slice);
    let interface_i_ptr = ast.find_typed_type::<Interface>("Test::I").unwrap();
    let interface_j_ptr = ast.find_typed_type::<Interface>("Test::J").unwrap();
    let interface_k_ptr = ast.find_typed_type::<Interface>("Test::K").unwrap();

    let interface_i_def = interface_i_ptr.borrow();
    let interface_j_def = interface_j_ptr.borrow();
    let interface_k_def = interface_k_ptr.borrow();

    let interface_k_bases = interface_k_def.base_interfaces();

    assert!(interface_i_def.base_interfaces().is_empty());
    assert!(interface_j_def.base_interfaces().is_empty());
    assert_eq!(interface_k_bases.len(), 2);

    assert_eq!(
        interface_k_bases[0].module_scoped_identifier(),
        interface_i_def.module_scoped_identifier(),
    );

    assert_eq!(
        interface_k_bases[1].module_scoped_identifier(),
        interface_j_def.module_scoped_identifier(),
    );
}

#[test]
fn must_inherit_from_interface() {
    let slice = "
    encoding = 1;
    module Test;
    class C {}
    interface I : C {}
    ";

    let error_reporter = parse_for_errors(slice);

    error_reporter.assert_errors(&["The Entity 'C' is not a valid type for this definition."])
}
