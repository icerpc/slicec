// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod encodings {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_errors;
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
            ",
            value = value,
        );

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter);
    }

    #[test]
    #[ignore = "The current message being emitted is not correct"]
    fn invalid_encodings_fail() {
        // Arrange
        let slice = "
            encoding = 3;
        ";

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, ["Unknown slice encoding version: 3"]);
    }

    #[test] // TODO: Maybe this shouldn't produce a parser error
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
