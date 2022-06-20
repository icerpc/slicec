// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::*;
use slice::grammar::*;

/// Verifies that exceptions can contain data members.
#[test]
fn can_contain_data_members() {
    // Arrange
    let slice = "
    module Test;
    exception E
    {
        i: int32,
        s: string,
        b: bool,
    }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let exception_ptr = ast.find_typed_type::<Exception>("Test::E").unwrap();
    let exception_def = exception_ptr.borrow();
    let data_members = exception_def.members();

    assert_eq!(data_members.len(), 3);
    assert!(matches!(data_members[0].identifier(), "i"));
    assert!(matches!(data_members[1].identifier(), "s"));
    assert!(matches!(data_members[2].identifier(), "b"));

    assert!(matches!(
        data_members[0].data_type.concrete_type(),
        Types::Primitive(Primitive::Int32),
    ));
    assert!(matches!(
        data_members[1].data_type.concrete_type(),
        Types::Primitive(Primitive::String),
    ));
    assert!(matches!(
        data_members[2].data_type.concrete_type(),
        Types::Primitive(Primitive::Bool),
    ));
}

/// Verifies that exceptions can be empty
#[test]
fn can_be_empty() {
    // Arrange
    let slice = "
    module Test;
    exception E {}
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let exception_ptr = ast.find_typed_type::<Exception>("Test::E").unwrap();
    let exception_def = exception_ptr.borrow();
    let data_members = exception_def.members();

    assert_eq!(data_members.len(), 0);
}

#[test]
fn cannot_redefine_data_members() {
    let slice = "
    encoding = 1;
    module Test;
    exception E
    {
        a: int32,
        a: string,
    }
    ";

    let error_reporter = parse_for_errors(slice);

    assert_errors!(error_reporter, ["redefinition of a", "a was previously defined here"]);
}
