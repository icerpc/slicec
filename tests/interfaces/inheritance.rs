// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::*;
use slice::grammar::*;

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
        interface I {}
        interface J {}
        interface K : I, J {}
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

    assert_errors!(error_reporter, &[
        "The Entity 'C' is not a valid type for this definition.",
    ]);
}

#[test]
fn inherits_correct_operations() {
    let slice = "
    module Test;
    interface A
    {
        opA();
    }
    interface B : A {
        opB();
    }
    interface C : A
    {
    }
    interface D : B, C
    {
        opD();
    }
";

    let ast = parse_for_ast(slice);
    let interface_a_ptr = ast.find_typed_type::<Interface>("Test::A").unwrap();
    let interface_b_ptr = ast.find_typed_type::<Interface>("Test::B").unwrap();
    let interface_d_ptr = ast.find_typed_type::<Interface>("Test::D").unwrap();

    let interface_a_def = interface_a_ptr.borrow();
    let interface_b_def = interface_b_ptr.borrow();
    let interface_d_def = interface_d_ptr.borrow();

    assert_eq!(interface_a_def.operations().len(), 1);
    assert_eq!(interface_a_def.all_inherited_operations().len(), 0);
    assert_eq!(interface_a_def.all_operations().len(), 1);
    assert_eq!(interface_a_def.operations()[0].identifier(), "opA");

    assert_eq!(interface_b_def.operations().len(), 1);
    assert_eq!(interface_b_def.all_inherited_operations().len(), 1);
    assert_eq!(interface_b_def.all_operations().len(), 2);
    assert_eq!(interface_b_def.operations()[0].identifier(), "opB");
    assert_eq!(
        interface_b_def.all_inherited_operations()[0].identifier(),
        "opA",
    );

    assert_eq!(interface_d_def.operations().len(), 1);
    assert_eq!(interface_d_def.all_inherited_operations().len(), 2);
    assert_eq!(interface_d_def.all_operations().len(), 3);
    assert_eq!(interface_d_def.operations()[0].identifier(), "opD");

    assert_eq!(
        interface_d_def.all_inherited_operations()[0].identifier(),
        "opA",
    );
    assert_eq!(
        interface_d_def.all_inherited_operations()[1].identifier(),
        "opB",
    );
}
