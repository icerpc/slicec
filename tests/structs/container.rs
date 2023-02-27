// Copyright (c) ZeroC, Inc.

mod structs {

    use crate::helpers::parsing_helpers::*;
    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::*;

    /// Verifies that structs can contain data members.
    #[test]
    fn can_contain_data_members() {
        // Arrange
        let slice = "
            module Test;

            struct S {
                i: int32,
                s: string,
                b: bool,
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let data_members = ast.find_element::<Struct>("Test::S").unwrap().members();

        assert_eq!(data_members.len(), 3);
        assert_eq!(data_members[0].identifier(), "i");
        assert_eq!(data_members[1].identifier(), "s");
        assert_eq!(data_members[2].identifier(), "b");
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
        let data_members = ast.find_element::<Struct>("Test::S").unwrap().members();
        assert_eq!(data_members.len(), 0);
    }

    #[test]
    fn cannot_redefine_data_members() {
        // Arrange
        let slice = "
            module Test;

            struct S {
                a: int32,
                a: string,
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::Redefinition {
            identifier: "a".to_owned(),
        })
        .add_note("'a' was previously defined here", None);

        check_diagnostics(diagnostics, [expected]);
    }
}

mod compact_structs {

    use crate::helpers::parsing_helpers::*;
    use slice::diagnostics::{Error, ErrorKind};
    /// Verifies that compact structs must contain at least one data member.
    #[test]
    fn must_not_be_empty() {
        // Arrange
        let slice = "
            module Test;

            compact struct S {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::CompactStructCannotBeEmpty);
        check_diagnostics(diagnostics, [expected]);
    }
}
