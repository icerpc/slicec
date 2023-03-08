// Copyright (c) ZeroC, Inc.

pub mod test_helpers;

use crate::test_helpers::*;
use slice::diagnostics::{Diagnostic, Error, ErrorKind};
use slice::grammar::Encoding;

#[test]
fn valid_mixed_encoding_works() {
    // Arrange
    let encoding1_slice = "
        encoding = 1
        module Test

        compact struct ACompactStruct {
            data: int32
        }

        enum AnEnum {
            A
            B
        }

        interface AnInterface {
            op() -> AnEnum
        }

        exception AnException {
            message: string
        }
    ";
    let encoding2_slice = "
        encoding = 2
        module Test
        struct AStruct {
            e: AnEnum
            i: AnInterface
            c: ACompactStruct
            ex: AnException
        }
    ";

    // Act
    let diagnostics = parse_multiple_for_diagnostics(&[encoding2_slice, encoding1_slice]);

    // Assert
    let expected: [Diagnostic; 0] = []; // Compiler needs the type hint.
    check_diagnostics(diagnostics, expected);
}

#[test]
fn invalid_mixed_encoding_fails() {
    // Arrange
    let encoding2_slice = "
        encoding = 2
        module Test

        custom ACustomType

        compact struct ACompactStruct {
            data: int32?
        }
    ";
    let encoding1_slice = "
        encoding = 1
        module Test
        compact struct AStruct {
            c: ACustomType
            s: ACompactStruct
        }
    ";

    // Act
    let diagnostics = parse_multiple_for_diagnostics(&[encoding1_slice, encoding2_slice]);

    // Assert
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
    check_diagnostics(diagnostics, expected);
}
