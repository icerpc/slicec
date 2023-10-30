// Copyright (c) ZeroC, Inc.

mod container;
mod mode_compatibility;

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};
use test_case::test_case;

#[test_case("int8"; "int8")]
#[test_case("uint8"; "uint8")]
#[test_case("int16"; "int16")]
#[test_case("uint16"; "uint16")]
#[test_case("int32"; "int32")]
#[test_case("uint32"; "uint32")]
#[test_case("varint32"; "varint32")]
#[test_case("varuint32"; "varuint32")]
#[test_case("int64"; "int64")]
#[test_case("uint64"; "uint64")]
#[test_case("varint62"; "varint62")]
#[test_case("varuint62"; "varuint62")]
fn supported_numeric_underlying_types_succeed(valid_type: &str) {
    // Arrange
    let slice = &format!(
        "
            module Test

            unchecked enum E : {valid_type} {{}}
        "
    );

    // Act/Assert
    assert_parses(slice);
}

#[test_case("string"; "string")]
#[test_case("float32"; "float32")]
#[test_case("float64"; "float64")]
fn invalid_underlying_type(underlying_type: &str) {
    // Arrange
    let slice = format!(
        "
            module Test
            enum E : {underlying_type} {{
                A
            }}
        "
    );

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::EnumUnderlyingTypeNotSupported {
        enum_identifier: "E".to_owned(),
        kind: Some(underlying_type.to_owned()),
    });
    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn optional_underlying_types_fail() {
    // Arrange
    let slice = "
        module Test

        enum E : int32? {
            A = 1
        }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::CannotUseOptionalUnderlyingType {
        enum_identifier: "E".to_owned(),
    });
    check_diagnostics(diagnostics, [expected]);
}
