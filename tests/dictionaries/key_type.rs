// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::{parse_for_diagnostics, pluralize_kind};
use slice::diagnostics::{Error, LogicKind, Note};
use test_case::test_case;

#[test]
fn optionals_are_disallowed() {
    // Arrange
    let slice = "
        module Test;
        typealias Dict = dictionary<int32?, int8>;
    ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(LogicKind::KeyMustBeNonOptional, None);
    assert_errors!(diagnostic_reporter, [&expected]);
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
            module Test;
            typealias Dict = dictionary<{key_type}, int8>;
        "
    );

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(diagnostic_reporter);
}

#[test_case("float32"; "float32")]
#[test_case("float64"; "float64")]
#[test_case("AnyClass"; "AnyClass")]
fn disallowed_primitive_types(key_type: &str) {
    // Arrange
    let slice = format!(
        "
            module Test;
            typealias Dict = dictionary<{key_type}, int8>;
        "
    );

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(LogicKind::KeyTypeNotSupported(key_type.to_owned()), None);
    assert_errors!(diagnostic_reporter, [&expected]);
}

#[test_case("sequence<int8>", "sequences" ; "sequences")]
#[test_case("dictionary<int8, bool>", "dictionaries" ; "dictionaries")]
fn collections_are_disallowed(key_type: &str, key_kind: &str) {
    // Arrange
    let slice = format!(
        "
            module Test;
            typealias Dict = dictionary<{key_type}, int8>;
        "
    );

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(LogicKind::KeyTypeNotSupported(key_kind.to_owned()), None);
    assert_errors!(diagnostic_reporter, [&expected]);
}

#[test_case("MyEnum", "unchecked enum MyEnum {}" ; "enums")]
#[test_case("MyCustom", "custom MyCustom;" ; "custom_types")]
fn allowed_constructed_types(key_type: &str, key_type_def: &str) {
    // Arrange
    let slice = format!(
        "
            module Test;
            {key_type_def}
            typealias Dict = dictionary<{key_type}, int8>;
        "
    );

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(diagnostic_reporter);
}

#[test_case("MyClass", "class MyClass {}", "class" ; "classes")]
#[test_case("MyException", "exception MyException {}", "exception" ; "exceptions")]
#[test_case("MyInterface", "interface MyInterface {}", "interface" ; "interfaces")]
#[test_case("MyTrait", "trait MyTrait;", "trait" ; "traits")]
fn disallowed_constructed_types(key_type: &str, key_type_def: &str, key_kind: &str) {
    // Arrange
    let file_encoding = if key_kind == "class" { "1" } else { "2" };
    let slice = format!(
        "
            encoding = {file_encoding};
            module Test;
            {key_type_def}
            typealias Dict = dictionary<{key_type}, int8>;
        "
    );

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new_with_notes(LogicKind::KeyTypeNotSupported(pluralize_kind(key_kind)), None, vec![
        Note::new(format!("{} '{}' is defined here:", key_kind, key_type), None),
    ]);
    assert_errors!(diagnostic_reporter, [&expected]);
}

#[test]
fn non_compact_structs_are_disallowed() {
    // Arrange
    let slice = "
        module Test;
        struct MyStruct {}
        typealias Dict = dictionary<MyStruct, int8>;
    ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new_with_notes(LogicKind::StructKeyMustBeCompact, None, vec![Note::new(
        "Struct 'MyStruct' is defined here:",
        None,
    )]);
    assert_errors!(diagnostic_reporter, [&expected]);
}

#[test]
fn compact_struct_with_allowed_members_is_allowed() {
    // Arrange
    let slice = "
        module Test;

        compact struct Inner
        {
            i32: int32,
        }

        compact struct Outer
        {
            b: bool,
            i: Inner,
        }

        typealias Dict = dictionary<Outer, int8>;
    ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(diagnostic_reporter);
}

#[test]
fn compact_struct_with_disallowed_members_is_disallowed() {
    // Arrange
    let slice = "
        module Test;

        compact struct Inner
        {
            i32: int32,
            f32: float32, // disallowed key type
        }

        compact struct Outer
        {
            seq: sequence<int8>, // disallowed key type
            i: Inner, // disallowed key type
            s: string,
        }

        typealias Dict = dictionary<Outer, int8>;
    ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected: [Error; 7] = [
        Error::new(LogicKind::KeyTypeNotSupported("sequences".to_owned()), None),
        Error::new(LogicKind::KeyTypeNotSupported("seq".to_owned()), None),
        Error::new(LogicKind::KeyTypeNotSupported("float32".to_owned()), None),
        Error::new(LogicKind::KeyTypeNotSupported("f32".to_owned()), None),
        Error::new_with_notes(
            LogicKind::StructKeyContainsDisallowedType("Inner".to_owned()),
            None,
            vec![Note::new("struct 'Inner' is defined here:", None)],
        ),
        Error::new(LogicKind::KeyTypeNotSupported("i".to_owned()), None),
        Error::new_with_notes(
            LogicKind::StructKeyContainsDisallowedType("Outer".to_owned()),
            None,
            vec![Note::new("struct 'Outer' is defined here:", None)],
        ),
    ];
    assert_errors!(diagnostic_reporter, expected);
}
