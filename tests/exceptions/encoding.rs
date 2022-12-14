// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice1 {

    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::Encoding;

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_diagnostics;

    /// Verifies that the slice parser with the Slice1 encoding emits errors when parsing an
    /// exception that is a data member.
    #[test]
    fn can_not_be_data_members() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;

            exception E
            {
            }

            compact struct S
            {
                e: E,
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::ExceptionNotSupported {
            encoding: Encoding::Slice1,
        })
        .add_note("file encoding was set to Slice1 here:", None);
        assert_errors!(diagnostic_reporter, [&expected]);
    }
}

mod slice2 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_diagnostics;
    use slice::compile_from_strings;
    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::Encoding;

    /// Verifies that the slice parser with the Slice2 encoding emits errors when parsing an
    /// exception that inherits from another exception.
    #[test]
    fn inheritance_fails() {
        // Arrange
        let slice = "
            module Test;

            exception A
            {
            }

            exception B : A
            {
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::NotSupportedWithEncoding {
            kind: "exception".to_owned(),
            identifier: "B".to_owned(),
            encoding: Encoding::Slice2,
        })
        .add_note("file is using the Slice2 encoding by default", None)
        .add_note(
            "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'",
            None,
        )
        .add_note("exception inheritance is only supported by the Slice1 encoding", None);

        assert_errors!(diagnostic_reporter, [&expected]);
    }

    /// Verifies that the slice parser with the Slice2 encoding does not emit errors when parsing
    /// exceptions that are data members.
    #[test]
    fn can_be_data_members() {
        // Arrange
        let slice = "
            module Test;

            exception E
            {
            }

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

    /// Verify that exceptions which are only Slice1 encodable a Slice2 operation.
    #[test]
    fn slice1_only_exceptions_cannot_be_thrown_from_slice2_operation() {
        // Arrange
        let slice1 = "
            encoding = 1;
            module Test;

            exception E
            {
                a: AnyClass,
            }
        ";

        let slice2 = "
            module Test;

            interface I
            {
                op() throws E;
            }
        ";

        // Act
        let diagnostic_reporter = compile_from_strings(&[slice1, slice2], None)
            .unwrap_err()
            .diagnostic_reporter;

        // Assert
        let expected = Error::new(ErrorKind::UnsupportedType {
            kind: "E".to_owned(),
            encoding: Encoding::Slice2,
        });
        assert_errors!(diagnostic_reporter, [&expected]);
    }

    #[test]
    fn cannot_throw_any_exception() {
        // Arrange
        let slice = "
            module Test;

            interface I
            {
                op() throws AnyException;
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::AnyExceptionNotSupported);
        assert_errors!(diagnostic_reporter, [&expected]);
    }
}
