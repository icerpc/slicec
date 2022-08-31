// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod encodings {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_diagnostics;
    use slice::parse_from_string;
    use test_case::test_case;

    /// Verifies that the supported encodings compile
    #[test_case("1"; "encoding 1")]
    #[test_case("2"; "encoding 2")]
    fn valid_encodings(value: &str) {
        // Arrange
        let slice = format!(
            "
                encoding = {value};
            "
        );

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(diagnostic_reporter);
    }

    #[test]
    #[ignore = "The current message being emitted is not correct"]
    fn invalid_encodings_fail() {
        // Arrange
        let slice = "
            encoding = 3;
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(diagnostic_reporter, ["Unknown slice encoding version: 3"]);
    }

    #[test]
    #[ignore = "No error message is being emitted"]
    fn encoding_must_be_first() {
        // Arrange
        let slice = "
            module Test;
            encoding = 2;
        ";

        // Act
        let error = parse_from_string(slice).err().is_some();

        // Assert
        assert!(error);
    }
}
