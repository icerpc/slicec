// Copyright (c) ZeroC, Inc.

mod structs {

    use slice::grammar::*;
    use slice::test_helpers::*;

    #[test]
    fn can_contain_tags() {
        // Arrange
        let slice = "
            module Test

            struct S {
                i: int32
                s: string
                tag(10) b: bool?
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let field = ast.find_element::<Field>("Test::S::b").unwrap();
        assert_eq!(field.tag(), Some(10));
    }
}

mod compact_structs {

    use slice::diagnostics::{Diagnostic, Error};
    use slice::test_helpers::*;

    #[test]
    fn cannot_contain_tags() {
        // Arrange
        let slice = "
            module Test

            compact struct S {
                i: int32
                s: string
                tag(10) b: bool?
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::CompactStructCannotContainTaggedFields)
            .add_note("struct 'S' is declared compact here", None);

        check_diagnostics(diagnostics, [expected]);
    }
}
