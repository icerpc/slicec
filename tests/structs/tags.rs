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
        let data_member_ptr = ast.find_typed_entity::<DataMember>("Test::S::b").unwrap();
        let data_member_tag = data_member_ptr.borrow().tag();

        assert_eq!(data_member_tag, Some(10));
    }
}

mod compact_structs {

    use crate::assert_errors;
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
        let expected_errors = [
            "tagged data members are not supported in compact structs. Consider removing the tag, or making the struct non-compact",
            "struct 'S' is declared compact here",
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, expected_errors);
    }
}
