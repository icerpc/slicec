// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};
use slicec::grammar::*;
use slicec::slice_file::Span;

#[test]
fn supports_single_inheritance() {
    // Arrange
    let slice = "
        mode = Slice1
        module Test
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
        mode = Slice1
        module Test

        class I {}

        class J {}

        class K : I, J {}
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::Syntax {
        message: "expected '{', but found ','".to_owned(),
    })
    .set_span(&Span::new((9, 20).into(), (9, 21).into(), "string-0"));

    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn field_shadowing_is_disallowed() {
    // Arrange
    let slice = "
        mode = Slice1
        module Test

        class I {
            i: int32
        }
        class J : I {
            i: int32
        }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::Shadows {
        identifier: "i".to_owned(),
    })
    .add_note("'i' was previously defined here", None);

    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn inherits_correct_fields() {
    // Arrange
    let slice = "
        mode = Slice1
        module Test

        class A {
            a: int32
        }
        class B : A {
            b: string
        }
        class C : B {
            c: float64
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let class_a_def = ast.find_element::<Class>("Test::A").unwrap();
    let class_b_def = ast.find_element::<Class>("Test::B").unwrap();
    let class_c_def = ast.find_element::<Class>("Test::C").unwrap();

    assert_eq!(class_a_def.fields().len(), 1);
    assert_eq!(class_a_def.all_fields().len(), 1);
    assert_eq!(class_a_def.all_fields()[0].identifier(), "a");

    assert_eq!(class_b_def.fields().len(), 1);
    assert_eq!(class_b_def.all_fields().len(), 2);
    assert_eq!(class_b_def.all_fields()[0].identifier(), "a");
    assert_eq!(class_b_def.all_fields()[1].identifier(), "b");

    assert_eq!(class_c_def.fields().len(), 1);
    assert_eq!(class_c_def.all_fields().len(), 3);
    assert_eq!(class_c_def.all_fields()[0].identifier(), "a");
    assert_eq!(class_c_def.all_fields()[1].identifier(), "b");
    assert_eq!(class_c_def.all_fields()[2].identifier(), "c");
}
