// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;
use slice::errors::{ErrorKind, LogicKind};
use slice::grammar::Encoding;
use slice::parse_from_strings;

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
    let parser_result = parse_from_strings(&[encoding2_slice, encoding1_slice]);

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
    let parser_result = parse_from_strings(&[encoding1_slice, encoding2_slice]);

    // Assert
    // TODO: we should provide a better error message to the user here
    let error_reporter = parser_result.err().unwrap().error_reporter;
    let expected = [
        LogicKind::UnsupportedType("ACustomType".to_owned(), Encoding::Slice1).into(),
        ErrorKind::new_note("file encoding was set to Slice1 here:"),
        LogicKind::UnsupportedType("ACompactStruct".to_owned(), Encoding::Slice1).into(),
        ErrorKind::new_note("file encoding was set to Slice1 here:"),
    ];

    assert_errors_new!(error_reporter, expected);
}
