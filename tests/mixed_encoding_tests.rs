// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;
use slice::diagnostics::{Error, ErrorKind, Note};
use slice::grammar::Encoding;
use slice::compile_from_strings;

#[test]
fn valid_mixed_encoding_works() {
    // Arrange
    let encoding1_slice = "
        encoding = 1;
        module Test;

        compact struct ACompactStruct
        {
            data: int32,
        }

        enum AnEnum
        {
            A,
            B,
        }

        interface AnInterface
        {
            op() -> AnEnum;
        }

        exception AnException
        {
            message: string,
        }
    ";
    let encoding2_slice = "
        encoding = 2;
        module Test;
        struct AStruct
        {
            e: AnEnum,
            i: AnInterface,
            c: ACompactStruct,
            ex: AnException,
        }
    ";

    // Act
    let parser_result = compile_from_strings(&[encoding2_slice, encoding1_slice], None);

    // Assert
    assert!(parser_result.ok().is_some());
}

#[test]
fn invalid_mixed_encoding_fails() {
    // Arrange
    let encoding2_slice = "
        encoding = 2;
        module Test;

        custom ACustomType;

        compact struct ACompactStruct
        {
            data: int32?,
        }
    ";
    let encoding1_slice = "
        encoding = 1;
        module Test;
        compact struct AStruct
        {
            c: ACustomType,
            s: ACompactStruct,
        }
    ";

    // Act
    let parser_result = compile_from_strings(&[encoding1_slice, encoding2_slice], None);

    // Assert
    // TODO: we should provide a better error message to the user here
    let diagnostic_reporter = parser_result.err().unwrap().diagnostic_reporter;
    let expected = [
        Error::new_with_notes(
            ErrorKind::UnsupportedType("ACustomType".to_owned(), Encoding::Slice1),
            None,
            vec![Note::new("file encoding was set to Slice1 here:", None)],
        ),
        Error::new_with_notes(
            ErrorKind::UnsupportedType("ACompactStruct".to_owned(), Encoding::Slice1),
            None,
            vec![Note::new("file encoding was set to Slice1 here:", None)],
        ),
    ];

    assert_errors!(diagnostic_reporter, expected);
}
