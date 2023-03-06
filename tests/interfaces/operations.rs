// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slice::diagnostics::{Error, ErrorKind};
use slice::grammar::*;
use test_case::test_case;

#[test]
fn can_have_no_parameters() {
    // Arrange
    let slice = "
        module Test

        interface I {
            op()
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
    assert!(operation.parameters().is_empty());
}

#[test]
fn can_have_no_return_type() {
    // Arrange
    let slice = "
        module Test

        interface I {
            op(a: int32)
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
    assert!(operation.return_members().is_empty());
}

#[test]
fn can_contain_tags() {
    // Arrange
    let slice = "
        module Test

        interface I {
            op(a: tag(1) int32?)
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
        module Test

        interface I {
            op(a: tag(1) int32?) -> tag(1) string?
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
    // Arrange
    let slice = "
        module Test

        interface I {
            op(a: int32, b: string, c: varuint62)
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
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
    // Arrange
    let slice = "
        module Test

        interface I {
            op() -> string
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
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
    // Arrange
    let slice = "
        module Test

        interface I {
            op() -> (r1: string, r2: bool)
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
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
fn operations_can_omit_throws_clause() {
    let slice = "
        module Test

        interface I {
            op()
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
    assert!(matches!(&operation.throws, Throws::None));
}

#[test]
fn operations_can_throw_specific_exceptions() {
    let slice = "
        module Test

        exception E {}

        interface I {
            op() throws E
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
    assert!(matches!(
        &operation.throws,
        Throws::Specific(exception_ref) if exception_ref.parser_scoped_identifier() == "Test::E",
    ));
}

#[test]
fn operations_can_only_throw_exceptions() {
    // Arrange
    let slice = "
        module Test

        struct S {}

        interface I {
            op() throws S
        }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::ConcreteTypeMismatch {
        expected: "exception".to_owned(),
        kind: "struct".to_owned(),
    });
    check_diagnostics(diagnostics, [expected]);
}

#[test_case("()"; "0 elements")]
#[test_case("(b: bool)"; "1 element")]
fn return_tuple_must_contain_two_or_more_elements(return_tuple: &str) {
    // Arrange
    let slice = format!(
        "
            module Test

            interface I {{
                op() -> {return_tuple}
            }}
        "
    );

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::ReturnTuplesMustContainAtLeastTwoElements);
    check_diagnostics(diagnostics, [expected]);
}

mod streams {
    use crate::test_helpers::*;
    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::*;

    #[test]
    fn can_have_streamed_parameter_and_return() {
        // Arrange
        let slice = "
            module Test

            interface I {
                op(a: stream uint32) -> stream uint32
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
        let parameters = operation.parameters();
        let returns = operation.return_members();

        assert!(parameters[0].is_streamed);
        assert!(returns[0].is_streamed);
    }

    #[test]
    fn operation_can_have_at_most_one_streamed_parameter() {
        // Arrange
        let slice = "
            module Test

            interface I {
                op(s: stream varuint62, s2: stream string)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = [
            Error::new(ErrorKind::StreamedMembersMustBeLast {
                parameter_identifier: "s".to_owned(),
            }),
            Error::new(ErrorKind::MultipleStreamedMembers),
        ];
        check_diagnostics(diagnostics, expected);
    }

    #[test]
    fn stream_parameter_must_be_last() {
        // Arrange
        let slice = "
            module Test

            interface I {
                op(s: stream varuint62, i: int32)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::StreamedMembersMustBeLast {
            parameter_identifier: "s".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }
}
