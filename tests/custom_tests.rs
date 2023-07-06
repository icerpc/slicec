// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod custom {

    use crate::test_helpers::*;
    use slicec::grammar::*;
    use test_case::test_case;

    #[test_case("Slice1")]
    #[test_case("Slice2")]
    fn type_parses(mode: &str) {
        // Arrange
        let slice = format!(
            "
                mode = {mode}
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
