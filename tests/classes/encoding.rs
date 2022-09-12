// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice2 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_diagnostics;
    use slice::diagnostics::{Diagnostic, LogicErrorKind, Note};
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
        let expected = [Diagnostic::new_with_notes(
            LogicErrorKind::NotSupportedWithEncoding("class".to_owned(), "C".to_owned(), Encoding::Slice2),
            None,
            vec![
                Note::new("file is using the Slice2 encoding by default", None),
                Note::new(
                    "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'",
                    None,
                ),
                Note::new("classes are only supported by the Slice1 encoding", None),
            ],
        )];
        assert_errors!(diagnostic_reporter, expected);
    }
}
