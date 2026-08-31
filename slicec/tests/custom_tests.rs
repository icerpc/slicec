// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod custom {

    use crate::test_helpers::*;
    use slicec::grammar::*;

    #[test]
    fn type_parses() {
        // Arrange
        let slice = "
            module Test
            custom ACustomType
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let custom_type = ast.find_symbol_by_id::<CustomType>("Test::ACustomType").unwrap();
        assert_eq!(custom_type.identifier(), "ACustomType");
    }
}
