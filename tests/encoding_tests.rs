// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod encodings {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_diagnostics;
    use slice::diagnostics::{Error, ErrorKind};
    use slice::parse_from_strings;
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
    fn invalid_encodings_fail() {
        // Arrange
        let slice = "
            encoding = 3;
        ";
        let expected = Error::new(ErrorKind::InvalidEncodingVersion(3), None);

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(diagnostic_reporter, [&expected]);
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
        let error = parse_from_strings(&[slice], None).err().is_some();

        // Assert
        assert!(error);
    }
}
