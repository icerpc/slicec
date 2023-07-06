// Copyright (c) ZeroC, Inc.

pub mod test_helpers;

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};
use slicec::grammar::Encoding;

#[test]
fn valid_mixed_encoding_works() {
    // Arrange
    let slice1 = "
        mode = Slice1
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
    let slice2 = "
        mode = Slice2
        module Test
        struct AStruct {
            e: AnEnum
            i: AnInterface
            c: ACompactStruct
            ex: AnException
        }
    ";

    // Act
    let diagnostics = parse_multiple_for_diagnostics(&[slice2, slice1]);

    // Assert
    let expected: [Diagnostic; 0] = []; // Compiler needs the type hint.
    check_diagnostics(diagnostics, expected);
}

#[test]
fn invalid_mixed_encoding_fails() {
    // Arrange
    let slice2 = "
        mode = Slice2
        module Test

        custom ACustomType

        compact struct ACompactStruct {
            data: int32?
        }
    ";
    let slice1 = "
        mode = Slice1
        module Test
        compact struct AStruct {
            c: ACustomType
            s: ACompactStruct
        }
    ";

    // Act
    let diagnostics = parse_multiple_for_diagnostics(&[slice1, slice2]);

    // Assert
    let expected = [
        Diagnostic::new(Error::UnsupportedType {
            kind: "ACustomType".to_owned(),
            encoding: Encoding::Slice1,
        }),
        Diagnostic::new(Error::UnsupportedType {
            kind: "ACompactStruct".to_owned(),
            encoding: Encoding::Slice1,
        }),
    ];
    check_diagnostics(diagnostics, expected);
}
