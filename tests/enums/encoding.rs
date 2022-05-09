// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice1 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_errors;

    /// Verifies that the slice parser with the Slice 1 encoding emits errors when parsing an enum
    /// that has an underlying type.
    #[test]
    fn underlying_types_fail() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            enum E : int32 {}
            ";
        let expected_errors = &[
            "enums with underlying types are not supported by the Slice 1 encoding",
            "file encoding was set to the Slice 1 encoding here:",
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, expected_errors);
    }
}

mod slice2 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_errors;
    use test_case::test_case;

    #[test_case("uint8")]
    #[test_case("int16")]
    #[test_case("uint16")]
    #[test_case("int32")]
    #[test_case("uint32")]
    fn supported_fixed_size_numeric_underlying_types_succeed(valid_type: &str) {
        // Arrange
        let slice = &format!(
            "
            module Test;
            enum E : {} {{}}
            ",
            valid_type,
        );

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter);
    }

    //
    /// * This test is passing currently, but Austin believes it results from the error checking for
    /// these types not being implemented yet.
    #[test_case("varint32")]
    #[test_case("varuint32")]
    #[test_case("varint62")]
    #[test_case("varuint62")]
    #[ignore]
    fn supported_variable_size_numeric_underlying_types_succeed(valid_type: &str) {
        // Arrange
        let slice = &format!(
            "
            module Test;
            enum E : {} {{}}
            ",
            valid_type,
        );

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter);
    }
}
