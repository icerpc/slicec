// Copyright (c) ZeroC, Inc. All rights reserved.

mod structs {

    use crate::assert_errors_new;
    use crate::helpers::parsing_helpers::*;
    use slice::errors::{ErrorKind, LogicKind};
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
            struct S
            {
                a: int32,
                a: string,
            }
        ";

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        let expected = [
            LogicKind::Redefinition("a".to_owned()).into(),
            ErrorKind::new_note("`a` was previously defined here".to_owned()),
        ];
        assert_errors_new!(error_reporter, expected);
    }
}

mod compact_structs {

    use crate::assert_errors_new;
    use crate::helpers::parsing_helpers::parse_for_errors;
    use slice::errors::{ErrorKind, LogicKind};
    /// Verifies that compact structs must contain at least one data member.
    #[test]
    fn must_not_be_empty() {
        // Arrange
        let slice = "
            module Test;
            compact struct S {}
        ";

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        let expected: ErrorKind = LogicKind::CompactStructIsEmpty.into();
        assert_errors_new!(error_reporter, [&expected]);
    }
}
