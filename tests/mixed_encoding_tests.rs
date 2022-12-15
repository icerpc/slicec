// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;
use slice::compile_from_strings;
use slice::diagnostics::{Error, ErrorKind};
use slice::grammar::Encoding;

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
    let diagnostic_reporter = parser_result.err().unwrap().diagnostic_reporter;
    let expected = [
        Error::new(ErrorKind::UnsupportedType {
            kind: "ACustomType".to_owned(),
            encoding: Encoding::Slice1,
        })
        .add_note("file encoding was set to Slice1 here:", None),
        Error::new(ErrorKind::UnsupportedType {
            kind: "ACompactStruct".to_owned(),
            encoding: Encoding::Slice1,
        })
        .add_note("file encoding was set to Slice1 here:", None),
    ];

    assert_errors!(diagnostic_reporter, expected);
}
