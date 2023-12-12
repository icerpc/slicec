// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slicec::diagnostics::{Diagnostic, Error};
use test_case::test_case;

#[test]
fn optional_keys_are_disallowed() {
    // Arrange
    let slice = "
        module Test
        typealias Dict = Dictionary<int32?, int8>
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::KeyMustBeNonOptional);
    check_diagnostics(diagnostics, [expected]);
}

#[test_case("bool"; "bool")]
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
#[test_case("string"; "string")]
fn allowed_primitive_types(key_type: &str) {
    // Arrange
    let slice = format!(
        "
            module Test
            typealias Dict = Dictionary<{key_type}, int8>
        "
    );

    // Act/Assert
    assert_parses(slice);
}

#[test_case("float32", "Slice2"; "float32")]
#[test_case("float64", "Slice2"; "float64")]
#[test_case("AnyClass", "Slice1"; "AnyClass")]
fn disallowed_primitive_types(key_type: &str, mode: &str) {
    // Arrange
    let slice = format!(
        "
            mode = {mode}
            module Test
            typealias Dict = Dictionary<{key_type}, uint8>
        "
    );

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::KeyTypeNotSupported {
        kind: key_type.to_owned(),
    });
    check_diagnostics(diagnostics, [expected]);
}

#[test_case("Sequence<int8>", "sequence" ; "sequence")]
#[test_case("Dictionary<int8, bool>", "dictionary" ; "dictionary")]
fn collections_are_disallowed(key_type: &str, key_kind: &str) {
    // Arrange
    let slice = format!(
        "
            module Test
            typealias Dict = Dictionary<{key_type}, int8>
        "
    );

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::KeyTypeNotSupported {
        kind: key_kind.to_owned(),
    });
    check_diagnostics(diagnostics, [expected]);
}

#[test_case("MyEnum", "enum MyEnum: int8 { A }" ; "simple enums")]
#[test_case("MyEnum", "unchecked enum MyEnum: int8 {}" ; "unchecked simple enums")]
#[test_case("MyCustom", "custom MyCustom" ; "custom_types")]
fn allowed_constructed_types(key_type: &str, key_type_def: &str) {
    // Arrange
    let slice = format!(
        "
            module Test
            {key_type_def}
            typealias Dict = Dictionary<{key_type}, int8>
        "
    );

    // Act/Assert
    assert_parses(slice);
}

#[test_case("MyEnum", "enum MyEnum { A }", "enum", "Slice2" ; "enums")]
#[test_case("MyEnum", "unchecked enum MyEnum {}", "enum", "Slice2" ; "unchecked enums")]
#[test_case("MyClass", "class MyClass {}", "class", "Slice1"; "classes")]
fn disallowed_constructed_types(key_type: &str, key_type_def: &str, key_kind: &str, mode: &str) {
    // Arrange
    let slice = format!(
        "
            mode = {mode}
            module Test

            {key_type_def}
            typealias Dict = Dictionary<{key_type}, uint8>
        "
    );

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::KeyTypeNotSupported {
        kind: format!("{key_kind} '{key_type}'"),
    });
    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn non_compact_structs_are_disallowed() {
    // Arrange
    let slice = "
        module Test

        struct MyStruct {}

        typealias Dict = Dictionary<MyStruct, int8>
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::StructKeyMustBeCompact);
    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn compact_struct_with_allowed_fields_is_allowed() {
    // Arrange
    let slice = "
        module Test

        compact struct Inner {
            i32: int32
        }

        compact struct Outer {
            b: bool
            i: Inner
        }

        typealias Dict = Dictionary<Outer, int8>
    ";

    // Act/Assert
    assert_parses(slice);
}

#[test]
fn compact_struct_with_disallowed_fields_is_disallowed() {
    // Arrange
    let slice = "
        module Test

        compact struct Inner {
            i32: int32
            f32: float32 // disallowed key type
        }

        compact struct Outer {
            seq: Sequence<int8> // disallowed key type
            i: Inner // disallowed key type
            s: string
        }

        typealias Dict = Dictionary<Outer, int8>
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::StructKeyContainsDisallowedType {
        struct_identifier: "Outer".to_owned(),
    })
    .add_note("invalid dictionary key type: sequence", None)
    .add_note(
        "struct 'Inner' contains fields that are not a valid dictionary key types",
        None,
    );

    check_diagnostics(diagnostics, [expected]);
}
