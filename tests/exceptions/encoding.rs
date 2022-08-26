// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice1 {

    use slice::diagnostics::{Diagnostic, LogicErrorKind, Note};
    use slice::grammar::Encoding;

    use crate::assert_errors_new;
    use crate::helpers::parsing_helpers::parse_for_diagnostics;

    /// Verifies that the slice parser with the Slice1 encoding emits errors when parsing an
    /// exception that is a data member.
    #[test]
    fn can_not_be_data_members() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            exception E {}
            compact struct S
            {
                e: E,
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic {
            diagnostic_kind: LogicErrorKind::ExceptionNotSupported(Encoding::Slice1).into(),
            span: None,
            notes: vec![Note::new("file encoding was set to Slice1 here:", None)],
        };
        assert_errors_new!(diagnostic_reporter, [&expected]);
    }
}

mod slice2 {

    use crate::helpers::parsing_helpers::parse_for_diagnostics;
    use crate::{assert_errors, assert_errors_new};
    use slice::diagnostics::{Diagnostic, LogicErrorKind, Note};
    use slice::grammar::Encoding;

    /// Verifies that the slice parser with the Slice2 encoding emits errors when parsing an
    /// exception that inherits from another exception.
    #[test]
    fn inheritance_fails() {
        // Arrange
        let slice = "
            module Test;
            exception A {}
            exception B : A {}
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic {
            diagnostic_kind: LogicErrorKind::NotSupportedWithEncoding(
                "exception".to_owned(),
                "B".to_owned(),
                Encoding::Slice2,
            )
            .into(),
            span: None,
            notes: vec![
                Note::new("file is using the Slice2 encoding by default", None),
                Note::new(
                    "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'",
                    None,
                ),
                Note::new("exception inheritance is only supported by the Slice1 encoding", None),
            ],
        };
        assert_errors_new!(diagnostic_reporter, [&expected]);
    }

    /// Verifies that the slice parser with the Slice2 encoding does not emit errors when parsing
    /// exceptions that are data members.
    #[test]
    fn can_be_data_members() {
        // Arrange
        let slice = "
            module Test;
            exception E {}
            struct S
            {
                e: E,
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(diagnostic_reporter);
    }
}
