// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod modes {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use test_case::test_case;

    /// Verifies that the supported modes compile
    #[test_case("Slice1")]
    #[test_case("Slice2")]
    fn valid_mode(value: &str) {
        // Arrange
        let slice = format!("mode = {value}");

        // Act/Assert
        assert_parses(slice);
    }

    #[test]
    fn invalid_modes_fail() {
        // Arrange
        let slice = "mode = Slice3";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::InvalidMode {
            mode: "Slice3".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn mode_must_be_first() {
        // Arrange
        let slice = "
            module Test
            mode = Slice2
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::Syntax{message: "expected one of '::', '[', 'class', 'compact', 'custom', 'doc comment', 'enum', 'exception', 'interface', 'struct', 'typealias', or 'unchecked', but found 'mode'".to_owned()});
        check_diagnostics(diagnostics, [expected]);
    }
}
