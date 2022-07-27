// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice1 {

    use slice::errors::*;

    use crate::assert_errors_new;
    use crate::helpers::parsing_helpers::parse_for_errors;

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
        let expected = [
            RuleKind::ExceptionNotSupported("1".to_owned()).into(),
            ErrorKind::Note("file encoding was set to Slice1 here:".to_owned()),
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors_new!(error_reporter, expected);
    }
}

mod slice2 {

    use crate::helpers::parsing_helpers::parse_for_errors;
    use crate::{assert_errors, assert_errors_new};
    use slice::errors::*;

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
        let expected = [
            RuleKind::NotSupportedWithEncoding("exception".to_owned(), "B".to_owned(), "2".to_owned()).into(),
            ErrorKind::Note("file is using the Slice2 encoding by default".to_owned()),
            ErrorKind::Note(
                "to use a different encoding, specify it at the top of the slice file\nex: 'encoding = 1;'".to_owned(),
            ),
            ErrorKind::Note("exception inheritance is only supported by the Slice1 encoding".to_owned()),
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors_new!(error_reporter, expected);
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
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter);
    }
}
