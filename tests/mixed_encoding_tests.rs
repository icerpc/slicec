// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;
use slice::errors::*;
use slice::parse_from_strings;

#[test]
fn valid_mixed_encoding_works() {
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

    assert!(parse_from_strings(&[encoding2_slice, encoding1_slice]).ok().is_some());
}

#[test]
fn invalid_mixed_encoding_fails() {
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
    let expected = [
        RuleKind::UnsupportedType("ACustomType".to_owned(), "1".to_owned()).into(),
        ErrorKind::create_note("file encoding was set to Slice1 here:"),
        RuleKind::UnsupportedType("ACompactStruct".to_owned(), "1".to_owned()).into(),
        ErrorKind::create_note("file encoding was set to Slice1 here:"),
    ];
    let error_reporter = parse_from_strings(&[encoding1_slice, encoding2_slice])
        .err()
        .unwrap()
        .error_reporter;

    // TODO: we should provide a better error message to the user here
    assert_errors_new!(error_reporter, expected);
}
