// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::parse_for_ast;
use slice::parse_from_string;

mod structs {

    use super::*;
    use slice::grammar::*;

    /// Verifies that structs can contain data members.
    #[test]
    fn can_contain_data_members() {
        // Arrange
        let slice = "
            module Test;
            struct S
            {
                i: int32,
                s: string,
                b: bool,
            }
            ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let struct_ptr = ast.find_typed_type::<Struct>("Test::S").unwrap();
        let struct_def = struct_ptr.borrow();
        let data_members = struct_def.members();

        assert_eq!(data_members.len(), 3);

        assert_eq!(data_members[0].identifier(), "i");
        assert_eq!(data_members[1].identifier(), "s");
        assert_eq!(data_members[2].identifier(), "b");

        assert!(matches!(
            data_members[0].data_type.concrete_type(),
            Types::Primitive(Primitive::Int32)
        ));
        assert!(matches!(
            data_members[1].data_type.concrete_type(),
            Types::Primitive(Primitive::String)
        ));
        assert!(matches!(
            data_members[2].data_type.concrete_type(),
            Types::Primitive(Primitive::Bool)
        ));
    }

    /// Verifies that structs can be empty
    #[test]
    fn can_be_empty() {
        // Arrange
        let slice = "
        module Test;
        struct S {}
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let struct_ptr = ast.find_typed_type::<Struct>("Test::S").unwrap();
        let struct_def = struct_ptr.borrow();
        let data_members = struct_def.members();

        assert_eq!(data_members.len(), 0);
    }
}

mod compact_structs {

    use super::*;

    /// Verifies that compact structs must contain at least one data member.
    #[test]
    fn must_not_be_empty() {
        // Arrange
        let slice = "
            module Test;
            compact struct S {}
        ";
        let expected_errors = &["compact structs must be non-empty"];

        // Act
        let (_, error_reporter) = parse_from_string(slice).ok().unwrap();

        // Assert
        error_reporter.assert_errors(expected_errors);
    }
}
