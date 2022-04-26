// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::parse_for_errors;

mod slice1 {

    use super::*;

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
        error_reporter.assert_errors(expected_errors);
    }
}

mod slice2 {

    use super::*;

    #[test]
    fn supported_fixed_size_numeric_underlying_types_succeed() {
        // Test case setup
        let valid_types = ["uint8", "int16", "uint16", "int32", "uint32"];

        // Run test for each case
        valid_types.iter().for_each(|valid_type| test(valid_type));

        fn test(valid_type: &str) {
            // Arrange
            let slice = &format!(
                "
                encoding = 2;
                module Test;
                enum E : {} {{}}
                ",
                valid_type,
            );
            let expected_errors = &[];

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            error_reporter.assert_errors(expected_errors);
        }
    }

    //
    /// * This test is passing currently, but Austin believes it results from the error checking for
    /// these types not being implemented yet.
    #[test]
    #[ignore]
    fn supported_variable_size_numeric_underlying_types_succeed() {
        // Test case setup
        let valid_types = ["varint32", "varuint32", "varint62", "varuint62"];

        // Run test for each case
        valid_types.iter().for_each(|valid_type| test(valid_type));

        fn test(valid_type: &str) {
            // Arrange
            let slice = &format!(
                "
                encoding = 2;
                module Test;
                enum E : {} {{}}
                ",
                valid_type,
            );
            let expected_errors = &[]; // TODO: Add the relevant error message once fixed

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            error_reporter.assert_errors(expected_errors);
        }
    }
}
