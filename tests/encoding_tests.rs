// Copyright (c) ZeroC, Inc.

pub mod helpers;

mod encodings {

    use crate::helpers::parsing_helpers::*;
    use slice::diagnostics::{Error, ErrorKind};
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

        // Act/Assert
        parse_for_ast(slice);
    }

    #[test]
    fn invalid_encodings_fail() {
        // Arrange
        let slice = "
            encoding = 3;
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::InvalidEncodingVersion { encoding: 3 });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn encoding_must_be_first() {
        // Arrange
        let slice = "
            module Test;
            encoding = 2;
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::Syntax{message: "expected one of '[', 'class', 'compact', 'custom', 'doc comment', 'enum', 'exception', 'interface', 'module', 'struct', 'typealias', or 'unchecked', but found 'encoding'".to_owned()});
        check_diagnostics(diagnostics, [expected]);
    }
}
