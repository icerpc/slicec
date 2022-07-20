// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors_new;
use crate::helpers::parsing_helpers::*;
use slice::errors::*;
use slice::grammar::*;

#[test]
fn can_have_no_parameters() {
    let slice = "
        module Test;
        interface I
        {
            op();
        }
    ";

    let ast = parse_for_ast(slice);

    let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
    assert!(operation.parameters().is_empty());
}

#[test]
fn can_have_no_return_type() {
    let slice = "
        module Test;
        interface I
        {
            op(a: int32);
        }
    ";

    let ast = parse_for_ast(slice);

    let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
    assert!(operation.return_members().is_empty());
}

#[test]
fn can_contain_tags() {
    // Arrange
    let slice = "
        module Test;
        interface I
        {
            op(a: tag(1) int32?);
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
    let tag_def = operation.parameters()[0].tag();
    assert_eq!(tag_def, Some(1));
}

#[test]
fn parameter_and_return_can_have_the_same_tag() {
    // Arrange
    let slice = "
        module Test;
        interface I
        {
            op(a: tag(1) int32?) -> tag(1) string?;
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
    let parameter_tag = operation.parameters()[0].tag();
    let return_tag = operation.return_members()[0].tag();
    assert_eq!(parameter_tag, Some(1));
    assert_eq!(return_tag, Some(1));
}

#[test]
fn can_have_parameters() {
    let slice = "
        module Test;
        interface I
        {
            op(a: int32, b: string, c: varuint62);
        }
    ";

    let ast = parse_for_ast(slice);

    let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
    let parameters = operation.parameters();

    assert_eq!(parameters.len(), 3);
    assert_eq!(parameters[0].identifier(), "a");
    assert_eq!(parameters[1].identifier(), "b");
    assert_eq!(parameters[2].identifier(), "c");

    assert!(matches!(
        parameters[0].data_type.concrete_type(),
        Types::Primitive(Primitive::Int32),
    ));
    assert!(matches!(
        parameters[1].data_type.concrete_type(),
        Types::Primitive(Primitive::String),
    ));
    assert!(matches!(
        parameters[2].data_type.concrete_type(),
        Types::Primitive(Primitive::VarUInt62),
    ));
}

#[test]
fn can_have_return_value() {
    let slice = "
        module Test;
        interface I
        {
            op() -> string;
        }
    ";

    let ast = parse_for_ast(slice);

    let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
    let returns = operation.return_members();

    assert_eq!(returns.len(), 1);
    assert_eq!(returns[0].identifier(), "returnValue");

    assert!(matches!(
        returns[0].data_type.concrete_type(),
        Types::Primitive(Primitive::String),
    ));
}

#[test]
fn can_have_return_tuple() {
    let slice = "
        module Test;
        interface I
        {
            op() -> (r1: string, r2: bool);
        }
    ";

    let ast = parse_for_ast(slice);

    let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
    let returns = operation.return_members();

    assert_eq!(returns.len(), 2);
    assert_eq!(returns[0].identifier(), "r1");
    assert_eq!(returns[1].identifier(), "r2");

    assert!(matches!(
        returns[0].data_type.concrete_type(),
        Types::Primitive(Primitive::String),
    ));
    assert!(matches!(
        returns[1].data_type.concrete_type(),
        Types::Primitive(Primitive::Bool),
    ));
}

#[test]
fn return_tuple_must_contain_two_or_more_elements() {
    let slice = "
        module Test;
        interface I
        {
            op() -> ();
        }
    ";
    let expected = RuleKind::InvalidParameter(
        "return tuple".to_owned(),
        InvalidParameterKind::ReturnTuplesMustContainAtleastTwoElements,
    );

    let error_reporter = parse_for_errors(slice);

    assert_errors_new!(error_reporter, [&expected]);
}

mod streams {
    use crate::assert_errors_new;
    use crate::helpers::parsing_helpers::*;
    use slice::errors::*;
    use slice::grammar::*;

    #[test]
    fn can_have_streamed_parameter_and_return() {
        let slice = "
            module Test;
            interface I
            {
                op(a: stream uint32) -> stream uint32;
            }
        ";

        let ast = parse_for_ast(slice);

        let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
        let parameters = operation.parameters();
        let returns = operation.return_members();

        assert!(parameters[0].is_streamed);
        assert!(returns[0].is_streamed);
    }

    #[test]
    fn operation_can_have_at_most_one_streamed_parameter() {
        let slice = "
            module Test;
            interface I
            {
                op(s: stream varuint62, s2: stream string);
            }
        ";
        let expected = RuleKind::InvalidParameter("s".to_owned(), InvalidParameterKind::StreamsMustBeLast);

        let error_reporter = parse_for_errors(slice);
        assert_errors_new!(error_reporter, [&expected]);
    }

    #[test]
    fn stream_parameter_must_be_last() {
        let slice = "
            module Test;
            interface I
            {
                op(s: stream varuint62, i: int32);
            }
        ";
        let expected = RuleKind::InvalidParameter("s".to_owned(), InvalidParameterKind::StreamsMustBeLast);

        let error_reporter = parse_for_errors(slice);
        assert_errors_new!(error_reporter, [&expected]);
    }
}
