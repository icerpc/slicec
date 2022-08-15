// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice2 {

    use crate::assert_errors_new;
    use crate::helpers::parsing_helpers::parse_for_diagnostics;
    use slice::diagnostics::{DiagnosticKind, LogicErrorKind};
    use slice::grammar::Encoding;

    #[test]
    fn unsupported_error() {
        // Arrange
        let slice = "
            module Test;
            class C {}
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = [
            LogicErrorKind::NotSupportedWithEncoding("class".to_owned(), "C".to_owned(), Encoding::Slice2).into(),
            DiagnosticKind::new_note("file is using the Slice2 encoding by default".to_owned()),
            DiagnosticKind::new_note(
                "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'".to_owned(),
            ),
            DiagnosticKind::new_note("classes are only supported by the Slice1 encoding".to_owned()),
        ];
        assert_errors_new!(diagnostic_reporter, expected);
    }
}
