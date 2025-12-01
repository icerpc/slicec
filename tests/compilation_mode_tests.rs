// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod compilation_mode {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error};
    use test_case::test_case;

    /// Verifies that the supported compilation modes compile
    #[test_case("Slice1")]
    #[test_case("Slice2")]
    fn valid_compilation_modes_succeed(value: &str) {
        // Arrange
        let slice = format!("mode = {value}");

        // Act/Assert
        assert_parses(slice);
    }

    #[test]
    fn invalid_compilation_modes_fail() {
        // Arrange
        let slice = "mode = Slice3";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::InvalidCompilationMode {
            mode: "Slice3".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn compilation_mode_must_appear_before_other_statements() {
        // Arrange
        let slice = "
            module Test
            mode = Slice2
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::Syntax{message: "expected one of 'doc comment', 'struct', 'exception', 'class', 'interface', 'enum', 'custom', 'typealias', 'compact', 'unchecked', '[', or '::', but found 'mode'".to_owned()});
        check_diagnostics(diagnostics, [expected]);
    }
}
