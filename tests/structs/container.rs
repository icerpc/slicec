// Copyright (c) ZeroC, Inc.

mod structs {

    use crate::test_helpers::*;
    use slice::diagnostics::{Diagnostic, Error};
    use slice::grammar::*;

    /// Verifies that structs can contain fields.
    #[test]
    fn can_contain_fields() {
        // Arrange
        let slice = "
            module Test

            struct S {
                i: int32
                s: string
                b: bool
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let fields = ast.find_element::<Struct>("Test::S").unwrap().fields();

        assert_eq!(fields.len(), 3);
        assert_eq!(fields[0].identifier(), "i");
        assert_eq!(fields[1].identifier(), "s");
        assert_eq!(fields[2].identifier(), "b");
        assert!(matches!(
            fields[0].data_type.concrete_type(),
            Types::Primitive(Primitive::Int32),
        ));
        assert!(matches!(
            fields[1].data_type.concrete_type(),
            Types::Primitive(Primitive::String),
        ));
        assert!(matches!(
            fields[2].data_type.concrete_type(),
            Types::Primitive(Primitive::Bool),
        ));
    }

    /// Verifies that structs can be empty
    #[test]
    fn can_be_empty() {
        // Arrange
        let slice = "
            module Test

            struct S {}
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let fields = ast.find_element::<Struct>("Test::S").unwrap().fields();
        assert_eq!(fields.len(), 0);
    }

    #[test]
    fn cannot_redefine_fields() {
        // Arrange
        let slice = "
            module Test

            struct S {
                a: int32
                a: string
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::Redefinition {
            identifier: "a".to_owned(),
        })
        .add_note("'a' was previously defined here", None);

        check_diagnostics(diagnostics, [expected]);
    }
}

mod compact_structs {

    use crate::test_helpers::*;
    use slice::diagnostics::{Diagnostic, Error};
    /// Verifies that compact structs must contain at least one field.
    #[test]
    fn must_not_be_empty() {
        // Arrange
        let slice = "
            module Test

            compact struct S {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::CompactStructCannotBeEmpty);
        check_diagnostics(diagnostics, [expected]);
    }
}
