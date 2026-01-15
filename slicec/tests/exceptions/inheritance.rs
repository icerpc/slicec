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
fn does_not_support_multiple_inheritance() {
    // Arrange
    let slice = "
        mode = Slice1
        module Test

        exception E1 {}

        exception E2 {}

        exception E3 : E1, E2 {}
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::Syntax {
        message: "expected '{', but found ','".to_owned(),
    })
    .set_span(&Span::new((9, 26).into(), (9, 27).into(), "string-0"));

    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn must_inherit_from_exception() {
    // Arrange
    let slice = "
        mode = Slice1
        module Test

        class C {}

        exception E : C {}
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::TypeMismatch {
        expected: "exception".to_owned(),
        actual: "class".to_owned(),
        is_concrete: true,
    });
    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn field_shadowing_is_disallowed() {
    // Arrange
    let slice = "
        mode = Slice1
        module Test

        exception I {
            i: int32
        }

        exception J : I {
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

        exception A {
            a: int32
        }

        exception B : A {
            b: string
        }

        exception C : B {
            c: float64
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let exception_a_def = ast.find_element::<Exception>("Test::A").unwrap();
    let exception_b_def = ast.find_element::<Exception>("Test::B").unwrap();
    let exception_c_def = ast.find_element::<Exception>("Test::C").unwrap();

    assert_eq!(exception_a_def.fields().len(), 1);
    assert_eq!(exception_a_def.all_fields().len(), 1);
    assert_eq!(exception_a_def.all_fields()[0].identifier(), "a");

    assert_eq!(exception_b_def.fields().len(), 1);
    assert_eq!(exception_b_def.all_fields().len(), 2);
    assert_eq!(exception_b_def.all_fields()[0].identifier(), "a");
    assert_eq!(exception_b_def.all_fields()[1].identifier(), "b");

    assert_eq!(exception_c_def.fields().len(), 1);
    assert_eq!(exception_c_def.all_fields().len(), 3);
    assert_eq!(exception_c_def.all_fields()[0].identifier(), "a");
    assert_eq!(exception_c_def.all_fields()[1].identifier(), "b");
    assert_eq!(exception_c_def.all_fields()[2].identifier(), "c");
}
