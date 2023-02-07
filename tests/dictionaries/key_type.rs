// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::{parse_for_diagnostics, pluralize_kind};
use slice::diagnostics::{Error, ErrorKind};
use test_case::test_case;

#[test]
fn optionals_are_disallowed() {
    // Arrange
    let slice = "
        module Test;
        typealias Dict = dictionary<int32?, int8>;
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::KeyMustBeNonOptional);
    assert_errors!(diagnostics, [&expected]);
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
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(diagnostics);
}

#[test_case("float32"; "float32")]
#[test_case("float64"; "float64")]
#[test_case("ServiceAddress"; "ServiceAddress")]
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
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::KeyTypeNotSupported {
        identifier: key_type.to_owned(),
    });
    assert_errors!(diagnostics, [&expected]);
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
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::KeyTypeNotSupported {
        identifier: key_kind.to_owned(),
    });
    assert_errors!(diagnostics, [&expected]);
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
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(diagnostics);
}

#[test_case("MyClass", "class MyClass {}", "class" ; "classes")]
#[test_case("MyException", "exception MyException {}", "exception" ; "exceptions")]
#[test_case("MyInterface", "interface MyInterface {}", "interface" ; "interfaces")]
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
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::KeyTypeNotSupported {
        identifier: pluralize_kind(key_kind),
    })
    .add_note(format!("{key_kind} '{key_type}' is defined here:"), None);

    assert_errors!(diagnostics, [&expected]);
}

#[test]
fn non_compact_structs_are_disallowed() {
    // Arrange
    let slice = "
        module Test;

        struct MyStruct
        {
        }

        typealias Dict = dictionary<MyStruct, int8>;
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::StructKeyMustBeCompact).add_note("Struct 'MyStruct' is defined here:", None);
    assert_errors!(diagnostics, [&expected]);
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
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(diagnostics);
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
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected: [Error; 7] = [
        Error::new(ErrorKind::KeyTypeNotSupported {
            identifier: "sequences".to_owned(),
        }),
        Error::new(ErrorKind::KeyTypeNotSupported {
            identifier: "seq".to_owned(),
        }),
        Error::new(ErrorKind::KeyTypeNotSupported {
            identifier: "float32".to_owned(),
        }),
        Error::new(ErrorKind::KeyTypeNotSupported {
            identifier: "f32".to_owned(),
        }),
        Error::new(ErrorKind::StructKeyContainsDisallowedType {
            struct_identifier: "Inner".to_owned(),
        })
        .add_note("struct 'Inner' is defined here:", None),
        Error::new(ErrorKind::KeyTypeNotSupported {
            identifier: "i".to_owned(),
        }),
        Error::new(ErrorKind::StructKeyContainsDisallowedType {
            struct_identifier: "Outer".to_owned(),
        })
        .add_note("struct 'Outer' is defined here:", None),
    ];
    assert_errors!(diagnostics, expected);
}
