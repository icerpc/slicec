// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

use helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;

mod sequences {

    use super::*;

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
        let seq_ptr = ast.find_typed_entity::<TypeAlias>("Test::Seq").unwrap();
        let seq_def = seq_ptr.borrow();
        let seq_type = seq_def.underlying.concrete_typeref();

        match seq_type {
            TypeRefs::Sequence(seq) => {
                matches!(
                    &seq.element_type.concrete_type(),
                    Types::Primitive(Primitive::Int8)
                );
            }
            _ => panic!("Expected sequence type"),
        }
    }
}
