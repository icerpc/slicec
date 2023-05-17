// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod custom {

    use crate::test_helpers::*;
    use slice::grammar::*;
    use test_case::test_case;

    #[test_case(1; "Slice1")]
    #[test_case(2; "Slice2")]
    fn type_parses(encoding: u8) {
        // Arrange
        let slice = format!(
            "
                encoding = Slice{encoding}
                module Test
                custom ACustomType
            "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let custom_type = ast.find_element::<CustomType>("Test::ACustomType").unwrap();
        assert_eq!(custom_type.identifier(), "ACustomType");
    }
}
