// Copyright (c) ZeroC, Inc. All rights reserved.

mod structs {

    use crate::helpers::parsing_helpers::*;
    use slice::grammar::*;

    #[test]
    fn can_contain_tags() {
        // Arrange
        let slice = "
            module Test;
            struct S {
                i: int32,
                s: string,
                b: tag(10) bool?,
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let data_member = ast.find_element::<DataMember>("Test::S::b").unwrap();
        assert_eq!(data_member.tag(), Some(10));
    }
}

mod compact_structs {

    use crate::assert_errors_new;
    use crate::helpers::parsing_helpers::*;
    use slice::errors::{ErrorKind, LogicKind};

    #[test]
    fn cannot_contain_tags() {
        // Arrange
        let slice = "
            module Test;
            compact struct S {
                i: int32,
                s: string,
                b: tag(10) bool?,
            }
        ";

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        let expected = [
            LogicKind::CompactStructCannotContainTaggedMembers.into(),
            ErrorKind::new_note("struct 'S' is declared compact here"),
        ];
        assert_errors_new!(error_reporter, expected);
    }
}
