// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::{parse_for_errors, pluralize_kind};
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
    assert_errors!(error_reporter, &[
        "invalid dictionary key type: optional types cannot be used as a dictionary key type",
    ]);
}

#[test_case("bool")]
#[test_case("int8")]
#[test_case("uint8")]
#[test_case("int16")]
#[test_case("uint16")]
#[test_case("int32")]
#[test_case("uint32")]
#[test_case("varint32")]
#[test_case("varuint32")]
#[test_case("int64")]
#[test_case("uint64")]
#[test_case("varint62")]
#[test_case("varuint62")]
#[test_case("string")]
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
    let error_reporter = parse_for_errors(&slice);

    // Assert
    assert_errors!(error_reporter);
}

#[test_case("float32")]
#[test_case("float64")]
#[test_case("AnyClass")]
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
    let error_reporter = parse_for_errors(&slice);

    // Assert
    assert_errors!(error_reporter, &[&*format!(
        "invalid dictionary key type: {} cannot be used as a dictionary key type",
        key_type,
    )]);
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
    let error_reporter = parse_for_errors(&slice);

    // Assert
    assert_errors!(error_reporter, &[&*format!(
        "invalid dictionary key type: {} cannot be used as a dictionary key type",
        key_kind,
    ),]);
}

#[test_case("MyEnum", "enum MyEnum {}" ; "enums")]
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
    let error_reporter = parse_for_errors(&slice);

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
    let error_reporter = parse_for_errors(&slice);

    // Assert
    assert_errors!(error_reporter, &[
        &*format!(
            "invalid dictionary key type: {} cannot be used as a dictionary key type",
            pluralize_kind(key_kind),
        ),
        &*format!("{} '{}' is defined here:", key_kind, key_type),
    ]);
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
    assert_errors!(error_reporter, &[
        "invalid dictionary key type: structs must be compact to be used as a dictionary key type",
        "struct 'MyStruct' is defined here:",
    ]);
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
    assert_errors!(error_reporter, &[
        "invalid dictionary key type: sequences cannot be used as a dictionary key type",
        "data member 'seq' cannot be used as a dictionary key type",

        "invalid dictionary key type: float32 cannot be used as a dictionary key type",
        "data member 'f32' cannot be used as a dictionary key type",

        "invalid dictionary key type: struct 'Inner' contains members that cannot be used as a dictionary key type",
        "struct 'Inner' is defined here:",

        "data member 'i' cannot be used as a dictionary key type",

        "invalid dictionary key type: struct 'Outer' contains members that cannot be used as a dictionary key type",
        "struct 'Outer' is defined here:",
    ]);
}
