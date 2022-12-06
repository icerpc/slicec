// Copyright (c) ZeroC, Inc. All rights reserved.

mod structs {

    use crate::helpers::parsing_helpers::*;
    use slice::grammar::*;

    #[test]
    fn can_contain_tags() {
        // Arrange
        let slice = "
            module Test;

            struct S
            {
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

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::*;
    use slice::diagnostics::{Error, ErrorKind};

    #[test]
    fn cannot_contain_tags() {
        // Arrange
        let slice = "
            module Test;

            compact struct S
            {
                i: int32,
                s: string,
                b: tag(10) bool?,
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::CompactStructCannotContainTaggedMembers)
            .add_note("struct 'S' is declared compact here", None);
        assert_errors!(diagnostic_reporter, [&expected]);
    }
}
