// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::{parse_for_errors, pluralize_kind};
use crate::{assert_errors, assert_errors_new};
use slice::errors::{ErrorKind, LogicKind};
use test_case::test_case;

#[test]
fn optionals_are_disallowed() {
    // Arrange
    let slice = "
        module Test;
        typealias Dict = dictionary<int32?, int8>;
    ";

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    let expected: ErrorKind = LogicKind::KeyMustBeNonOptional.into();
    assert_errors_new!(error_reporter, [&expected]);
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
            typealias Dict = dictionary<{}, int8>;
        ",
        key_type,
    );

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors!(error_reporter);
}

#[test_case("float32"; "float32")]
#[test_case("float64"; "float64")]
#[test_case("AnyClass"; "AnyClass")]
fn disallowed_primitive_types(key_type: &str) {
    // Arrange
    let slice = format!(
        "
            module Test;
            typealias Dict = dictionary<{}, int8>;
        ",
        key_type,
    );

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    let expected: ErrorKind = LogicKind::KeyTypeNotSupported(key_type.to_owned()).into();
    assert_errors_new!(error_reporter, [&expected]);
}

#[test_case("sequence<int8>", "sequences" ; "sequences")]
#[test_case("dictionary<int8, bool>", "dictionaries" ; "dictionaries")]
fn collections_are_disallowed(key_type: &str, key_kind: &str) {
    // Arrange
    let slice = format!(
        "
            module Test;
            typealias Dict = dictionary<{}, int8>;
        ",
        key_type,
    );

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    let expected: ErrorKind = LogicKind::KeyTypeNotSupported(key_kind.to_owned()).into();
    assert_errors_new!(error_reporter, [&expected]);
}

#[test_case("MyEnum", "unchecked enum MyEnum {}" ; "enums")]
#[test_case("MyCustom", "custom MyCustom;" ; "custom_types")]
fn allowed_constructed_types(key_type: &str, key_type_def: &str) {
    // Arrange
    let slice = format!(
        "
            module Test;
            {}
            typealias Dict = dictionary<{}, int8>;
        ",
        key_type_def, key_type,
    );

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors!(error_reporter);
}

#[test_case("MyClass", "class MyClass {}", "class" ; "classes")]
#[test_case("MyException", "exception MyException {}", "exception" ; "exceptions")]
#[test_case("MyInterface", "interface MyInterface {}", "interface" ; "interfaces")]
#[test_case("MyTrait", "trait MyTrait;", "trait" ; "traits")]
fn disallowed_constructed_types(key_type: &str, key_type_def: &str, key_kind: &str) {
    // Arrange
    let slice = format!(
        "
            encoding = {file_encoding};
            module Test;
            {key_type_definition}
            typealias Dict = dictionary<{key_type}, int8>;
        ",
        file_encoding = if key_kind == "class" { "1" } else { "2" },
        key_type_definition = key_type_def,
        key_type = key_type,
    );

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    let expected: [ErrorKind; 2] = [
        LogicKind::KeyTypeNotSupported(pluralize_kind(key_kind)).into(),
        ErrorKind::new_note(format!("{} '{}' is defined here:", key_kind, key_type)),
    ];
    assert_errors_new!(error_reporter, expected);
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
    let error_reporter = parse_for_errors(slice);

    // Assert
    let expected: [ErrorKind; 2] = [
        LogicKind::StructKeyMustBeCompact.into(),
        ErrorKind::new_note("struct 'MyStruct' is defined here:".to_owned()),
    ];
    assert_errors_new!(error_reporter, expected);
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
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors!(error_reporter);
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
    let error_reporter = parse_for_errors(slice);

    // Assert
    let expected: [ErrorKind; 9] = [
        LogicKind::KeyTypeNotSupported("sequences".to_owned()).into(),
        LogicKind::KeyTypeNotSupported("seq".to_owned()).into(),
        LogicKind::KeyTypeNotSupported("float32".to_owned()).into(),
        LogicKind::KeyTypeNotSupported("f32".to_owned()).into(),
        LogicKind::StructKeyContainsDisallowedType("Inner".to_owned()).into(),
        ErrorKind::new_note("struct 'Inner' is defined here:"),
        LogicKind::KeyTypeNotSupported("i".to_owned()).into(),
        LogicKind::StructKeyContainsDisallowedType("Outer".to_owned()).into(),
        ErrorKind::new_note("struct 'Outer' is defined here:".to_owned()),
    ];
    assert_errors_new!(error_reporter, expected);
}
