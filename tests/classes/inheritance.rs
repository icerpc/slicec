// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::*;
use crate::{assert_errors, assert_errors_new};
use slice::errors::*;
use slice::grammar::*;

#[test]
fn supports_single_inheritance() {
    // Arrange
    let slice = "
        encoding = 1;
        module Test;
        class I {}
        class J : I {}
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let class_i_def = ast.find_element::<Class>("Test::I").unwrap();
    let class_j_def = ast.find_element::<Class>("Test::J").unwrap();

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
    let expected: [&dyn ErrorType; 2] = [
        &RuleKind::from(InvalidIdentifierKind::Shadows("i".to_owned())),
        &Note::new("`i` was previously defined here"),
    ];

    // Act
    let error_reporter = parse_for_errors(slice);

    assert_errors_new!(error_reporter, expected);
}

#[test]
fn inherits_correct_data_members() {
    // Arrange
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

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let class_a_def = ast.find_element::<Class>("Test::A").unwrap();
    let class_b_def = ast.find_element::<Class>("Test::B").unwrap();
    let class_c_def = ast.find_element::<Class>("Test::C").unwrap();

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
