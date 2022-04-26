// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::parse_for_ast;

mod slice2 {

    #[test]
    fn test_allowed_primitive_as_key() {
        // Test case setup
        let valid_types = [
            "uint8",
            "uint16",
            "uint32",
            "uint64",
            "int8",
            "int16",
            "int32",
            "int64",
            "varint32",
            "varuint32",
            "varint62",
            "varuint62",
            "string",
            "bool",
        ];
        valid_types.iter().for_each(|valid_type| test(valid_type));

        fn test(key_type: &str) {
            // Arrange
            let slice = format!(
                "
                encoding = 2;
                module Test;
                typealias MyDict = dictionary<{}, int32>;
                ",
                key_type
            );

            // Act
            let ast = parse_for_ast(&slice);

            // Assert
            assert!(ast.errors.is_empty());
        }
    }

    #[test]
    fn test_compact_struct_containing_allowed_primitves_as_key() {
        // Arrange
        let slice = "
            encoding = 2;
            module Test;
            compact struct S {
                a : uint8;
                b : uint16;
                c : uint32;
                d : uint64;
                e : int8;
                f : int16;
                g : int32;
                h : int64;
                i : varint32;
                j : varuint32;
                k : varint62;
                l : varuint62;
                m : string;
                n : bool;
            }
            typealias MyDict = dictionary<S, int32>;
            ";

        // Act
        let ast = parse_for_ast(&slice);

        // Assert
        assert!(ast.errors.is_empty());
    }
}
