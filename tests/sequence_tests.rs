// Copyright (c) ZeroC, Inc.

pub mod helpers;

mod sequences {

    use crate::helpers::parsing_helpers::parse_for_ast;
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
}
