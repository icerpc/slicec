// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::*;
use crate::{assert_errors, assert_errors_new};
use slice::errors::{ErrorKind, LogicKind};
use slice::grammar::*;

#[test]
fn supports_single_inheritance() {
    // Arrange
    let slice = "
        encoding = 1;
        module Test;
        exception E1 {}
        exception E2 : E1 {}
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let e2_def = ast.find_element::<Exception>("Test::E2").unwrap();
    assert_eq!(e2_def.base_exception().unwrap().module_scoped_identifier(), "Test::E1");
}

#[test]
#[ignore = "reason: TODO Need to update AST Error emission"]
fn does_not_support_multiple_inheritance() {
    // Arrange
    let slice = "
        encoding = 1;
        module Test;
        exception E1 {}
        exception E2 {}
        exception E3 : E1, E2 {}
    ";

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    let expected: ErrorKind = LogicKind::CanOnlyInheritFromSingleBase("exception".to_string()).into();
    assert_errors_new!(error_reporter, [&expected]);
}

#[test]
#[ignore = "reason: TODO Need to update AST Error emission"]
fn must_inherit_from_exception() {
    // Arrange
    let slice = "
        encoding = 1;
        module Test;
        class C {}
        exception E : C {}
    ";

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors!(error_reporter, [
        "type mismatch: expected an exception but found a class",
    ]);
}

#[test]
fn data_member_shadowing_is_disallowed() {
    // Arrange
    let slice = "
        encoding = 1;
        module Test;
        exception I
        {
            i: int32
        }
        exception J : I
        {
            i: int32
        }
    ";

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    let expected = [
        LogicKind::Shadows("i".to_owned()).into(),
        ErrorKind::new_note("`i` was previously defined here".to_owned()),
    ];
    assert_errors_new!(error_reporter, expected);
}

#[test]
fn inherits_correct_data_members() {
    // Arrange
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

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let exception_a_def = ast.find_element::<Exception>("Test::A").unwrap();
    let exception_b_def = ast.find_element::<Exception>("Test::B").unwrap();
    let exception_c_def = ast.find_element::<Exception>("Test::C").unwrap();

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
