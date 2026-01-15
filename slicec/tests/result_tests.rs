// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod results {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use slicec::grammar::*;
    use slicec::slice_file::Span;

    #[test]
    fn are_parsed_correctly() {
        // Arrange
        let slice = "
            module Test
            interface I {
                op() -> Result<Sequence<uint8>, Error>
            }

            unchecked enum Error {}
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
        let returns = operation.return_members();

        assert_eq!(returns.len(), 1);
        let return_type = returns[0].data_type();
        assert_eq!(
            *return_type.span(),
            Span::new((4, 25).into(), (4, 55).into(), "string-0"),
        );

        let Types::ResultType(result_type) = return_type.concrete_type() else { panic!() };

        assert_eq!(result_type.success_type.type_string(), "Sequence<uint8>");
        assert_eq!(
            *result_type.success_type.span(),
            Span::new((4, 32).into(), (4, 47).into(), "string-0"),
        );

        assert_eq!(result_type.failure_type.type_string(), "Error");
        assert_eq!(
            *result_type.failure_type.span(),
            Span::new((4, 49).into(), (4, 54).into(), "string-0"),
        );
    }

    #[test]
    fn can_be_nested() {
        // Arrange
        let slice = "
            module Test
            interface I {
                op() -> Result<Result<bool, string>, string>
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
        let returns = operation.return_members();
        assert_eq!(returns.len(), 1);
        let Types::ResultType(result_type) = returns[0].data_type().concrete_type() else { panic!() };

        let Types::ResultType(inner_result_type) = result_type.success_type.concrete_type() else { panic!() };
        assert!(matches!(
            result_type.failure_type.concrete_type(),
            Types::Primitive(Primitive::String),
        ));

        assert!(matches!(
            inner_result_type.success_type.concrete_type(),
            Types::Primitive(Primitive::Bool),
        ));
        assert!(matches!(
            inner_result_type.failure_type.concrete_type(),
            Types::Primitive(Primitive::String),
        ));
    }

    #[test]
    fn are_disallowed_in_slice1_mode() {
        // Arrange
        let slice = "
            mode = Slice1
            module Test

            typealias R = Result<string, bool>
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::UnsupportedType {
            kind: "Result<string, bool>".to_owned(),
            mode: Encoding::Slice1,
        })
        .add_note("'Result' can only be used in Slice2 mode", None);
        check_diagnostics(diagnostics, [expected]);
    }
}
