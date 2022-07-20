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
        let ast = parse_for_ast(slice);

        // Assert
        let data_member = ast.find_element::<DataMember>("Test::S::b").unwrap();
        assert_eq!(data_member.tag(), Some(10));
    }
}

mod compact_structs {

    use slice::errors::*;

    use crate::assert_errors_new;
    use crate::helpers::parsing_helpers::*;

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
        let expected: [&dyn ErrorType; 2] = [
            &RuleKind::InvalidMember(
                "b".to_owned(),
                InvalidMemberKind::TaggedDataMemberNotSupportedInCompactStructs,
            ),
            &Note::new("struct 'S' is declared compact here"),
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors_new!(error_reporter, expected);
    }
}
