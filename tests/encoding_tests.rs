// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod encodings {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_diagnostics;
    use slice::diagnostics::{ErrorBuilder, ErrorKind};
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

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = [ErrorBuilder::new(ErrorKind::InvalidEncodingVersion(3)).build()];
        assert_errors!(diagnostic_reporter, expected);
    }

    #[test]
    fn encoding_must_be_first() {
        // Arrange
        let slice = "
            module Test;
            encoding = 2;
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = ErrorBuilder::new(ErrorKind::Syntax("expected one of \"[\", class_keyword, compact_keyword, custom_keyword, doc_comment, enum_keyword, exception_keyword, interface_keyword, module_keyword, struct_keyword, type_alias_keyword, unchecked_keyword, but found 'EncodingKeyword'".to_owned())).build();
        assert_errors!(diagnostic_reporter, [&expected]);
    }
}
