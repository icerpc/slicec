// Copyright (c) ZeroC, Inc.

pub mod helpers;

mod sequences {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_diagnostics};
    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::*;

    #[test]
    fn can_contain_primitive_types() {
        // Arrange
        let slice = "
            module Test;
            typealias Seq = sequence<int8>;
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let seq_def = ast.find_element::<TypeAlias>("Test::Seq").unwrap();
        let seq_type = seq_def.underlying.concrete_typeref();

        match seq_type {
            TypeRefs::Sequence(seq) => assert!(matches!(
                &seq.element_type.concrete_type(),
                Types::Primitive(Primitive::Int8)
            )),
            _ => panic!("Expected TypeRefs<Sequence>"),
        }
    }

    #[test]
    fn sequences_containing_dictionaries_get_validated() {
        // Arrange
        let slice = "
            module Test;
            typealias Seq = sequence<dictionary<int32, dictionary<float32, float32>>>;
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::KeyTypeNotSupported {
            identifier: "float32".to_owned(),
        });
        assert_errors!(diagnostics, [&expected]);
    }
}
