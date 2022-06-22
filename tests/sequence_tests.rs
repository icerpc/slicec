// Copyright (c) ZeroC, Inc. All rights reserved.

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

        if let TypeRefs::Sequence(seq) = seq_type {
            matches!(&seq.element_type.concrete_type(), Types::Primitive(Primitive::Int8));
        } else {
            panic!("Expected sequence type");
        }
    }
}
