// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::*;
use slice::diagnostics::{Error, ErrorKind};
use slice::grammar::*;
use slice::slice_file::Span;

#[test]
fn supports_single_inheritance() {
    // Arrange
    let slice = "
        encoding = 1;
        module Test;

        exception E1
        {
        }

        exception E2 : E1
        {
        }
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
        encoding = 1;
        module Test;

        exception E1
        {
        }

        exception E2
        {
        }

        exception E3 : E1, E2
        {
        }
    ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::Syntax("expected one of \"{\", but found 'Comma'".to_owned()))
        .set_span(&Span::new((13, 20).into(), (13, 21).into(), "string-0"));

    assert_errors!(diagnostic_reporter, [&expected]);
}

#[test]
fn must_inherit_from_exception() {
    // Arrange
    let slice = "
        encoding = 1;
        module Test;

        class C
        {
        }

        exception E : C
        {
        }
    ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(diagnostic_reporter, [
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
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::Shadows("i".to_owned())).add_note("`i` was previously defined here", None);
    assert_errors!(diagnostic_reporter, [&expected]);
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
