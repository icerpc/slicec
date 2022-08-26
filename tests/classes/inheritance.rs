// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors_new;
use crate::helpers::parsing_helpers::*;
use slice::diagnostics::{Diagnostic, DiagnosticKind, LogicErrorKind, Note};
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
    // Arrange
    let slice = "
        encoding = 1;
        module Test;
        class I {}
        class J {}
        class K : I, J {}
    ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected: DiagnosticKind = LogicErrorKind::CanOnlyInheritFromSingleBase("class".to_owned()).into();
    assert_errors_new!(diagnostic_reporter, [&expected]);
}

#[test]
fn data_member_shadowing_is_disallowed() {
    // Arrange
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

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = [Diagnostic {
        diagnostic_kind: LogicErrorKind::Shadows("i".to_owned()).into(),
        span: None,
        notes: vec![Note::new("`i` was previously defined here", None)],
    }];
    assert_errors_new!(diagnostic_reporter, expected);
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
