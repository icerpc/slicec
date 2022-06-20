// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::*;
use slice::grammar::*;

#[test]
fn supports_single_inheritance() {
    let slice = "
    encoding = 1;
    module Test;
    class I {}
    class J : I {}
    ";

    let ast = parse_for_ast(slice);
    let class_i_ptr = ast.find_typed_type::<Class>("Test::I").unwrap();
    let class_j_ptr = ast.find_typed_type::<Class>("Test::J").unwrap();
    let class_i_def = class_i_ptr.borrow();
    let class_j_def = class_j_ptr.borrow();

    assert!(class_i_def.base_class().is_none());
    assert!(class_j_def.base_class().is_some());

    assert_eq!(
        class_j_def.base_class().unwrap().module_scoped_identifier(),
        class_i_def.module_scoped_identifier(),
    );
}

#[test]
fn does_not_support_multiple_inheritance() {
    let slice = "
    encoding = 1;
    module Test;
    class I {}
    class J {}
    class K : I, J {}
    ";

    let error_reporter = parse_for_errors(slice);

    assert_errors!(error_reporter, ["classes can only inherit from a single base class",]);
}

#[test]
fn data_member_shadowing_is_disallowed() {
    let slice = "
    encoding = 1;
    module Test;
    class I
    {
        i: int32
    }
    class J : I
    {
        i: int32
    }
    ";

    let error_reporter = parse_for_errors(slice);

    assert_errors!(error_reporter, [
        "i shadows another symbol",
        "i was previously defined here"
    ]);
}

#[test]
fn inherits_correct_data_members() {
    let slice = "
    encoding = 1;
    module Test;
    class A
    {
        a: int32
    }
    class B : A
    {
        b: string
    }
    class C : B
    {
        c: float64
    }
    ";

    let ast = parse_for_ast(slice);
    let class_a_ptr = ast.find_typed_type::<Class>("Test::A").unwrap();
    let class_b_ptr = ast.find_typed_type::<Class>("Test::B").unwrap();
    let class_c_ptr = ast.find_typed_type::<Class>("Test::C").unwrap();

    let class_a_def = class_a_ptr.borrow();
    let class_b_def = class_b_ptr.borrow();
    let class_c_def = class_c_ptr.borrow();

    assert_eq!(class_a_def.members().len(), 1);
    assert_eq!(class_a_def.all_members().len(), 1);
    assert_eq!(class_a_def.all_members()[0].identifier(), "a");

    assert_eq!(class_b_def.members().len(), 1);
    assert_eq!(class_b_def.all_members().len(), 2);
    assert_eq!(class_b_def.all_members()[0].identifier(), "a");
    assert_eq!(class_b_def.all_members()[1].identifier(), "b");

    assert_eq!(class_c_def.members().len(), 1);
    assert_eq!(class_c_def.all_members().len(), 3);
    assert_eq!(class_c_def.all_members()[0].identifier(), "a");
    assert_eq!(class_c_def.all_members()[1].identifier(), "b");
    assert_eq!(class_c_def.all_members()[2].identifier(), "c");
}
