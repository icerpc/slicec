// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::parse_for_errors;

mod slice1 {}

mod slice2 {

    use super::*;

    /// Invalid Simple dictionary key types test.
    macro_rules! test_simple_dictionary_key_types_fails {
        ($(($test_name:ident, $key_type:expr)),*) => {
            $(
                #[test]
                #[ignore] // Remove ignore when disallowed key errors are added.
                fn $test_name() {
                    // Arrange
                    let slice = &format!(
                        "
                        encoding = 2;
                        module Test;
                        typealias TestDict = dictionary<{key_type}, int32>;
                        ",
                        key_type=$key_type
                    );
                    let expected_errors = &[&format!(
                        "{} cannot be used as a dictionary key type",
                        $key_type,
                    ),
                    "'TestDict' was defined here:",
                    ]; // Add the expected errors here when disallowed key errors are added.

                    // Act
                    let error_reporter = parse_for_errors(slice);

                    // Assert
                    error_reporter.assert_errors(expected_errors);
                }
            )*
        }
    }

    // Invalid Simple dictionary key types test cases.
    test_simple_dictionary_key_types_fails!(
        (optional_uint8_as_key_invalid, "uint8?"),
        (optional_uint16_as_key_invalid, "uint16?"),
        (optional_uint32_as_key_invalid, "uint32?"),
        (optional_uint64_as_key_invalid, "uint64?"),
        (optional_int8_as_key_invalid, "int8?"),
        (optional_int16_as_key_invalid, "int16?"),
        (optional_int32_as_key_invalid, "int32?"),
        (optional_int64_as_key_invalid, "int64?"),
        (optional_varint32_as_key_invalid, "varint32?"),
        (optional_varuint32_as_key_invalid, "varuint32?"),
        (optional_varint62_as_key_invalid, "varint62?"),
        (optional_varuint62_as_key_invalid, "varuint62?"),
        (optional_string_as_key_invalid, "string?"),
        (optional_bool_as_key_invalid, "bool?"),
        (optional_sequence_as_key_invalid, "sequence<int32>?"),
        (proxy_as_key_invalid, "proxy"),
        (float32_as_key_invalid, "float32"),
        (float64_as_key_invalid, "float64")
    );

    /// Invalid Constructed dictionary key types test.
    macro_rules! test_constructed_dictionary_key_types_fails {
        ($(($test_name:ident, $key_type:expr, $key_ident:expr, $key_def:expr)),*) => {
            $(
                #[test]
                #[ignore] // Remove ignore when disallowed key errors are added.
                fn $test_name() {
                    // Arrange
                    let slice = &format!("
                        encoding = 2;
                        module test;

                        {key_def}

                        typealias TestDict = dictionary<{key_type}, int32>;
                        ",
                        key_def = $key_def,
                        key_type = $key_ident,
                    );

                    let expected_errors: [&str; 2] = [
                        &format!(
                            "{} '{}' cannot be used as a dictionary key type",
                            $key_type,
                            $key_ident,
                        ),
                        &format!(
                            "{} was defined here:",
                            $key_ident,
                        ),
                    ];
                    // Act
                    let error_reporter = parse_for_errors(slice);

                    // Assert
                    error_reporter.assert_errors(&expected_errors);
                }
            )*
        }
    }

    // Invalid Constructed dictionary key types test cases.
    test_constructed_dictionary_key_types_fails!(
        (
            dictionary_as_key_invalid,
            "dictionary",
            "MyDict",
            "typealias MyDict = dictionary<int32, int32>"
        ),
        (
            struct_as_key_invalid,
            "struct",
            "MyStruct",
            "struct MyStruct { }"
        ),
        (
            struct_with_tags_as_key_invalid,
            "struct",
            "MyTaggedStruct",
            "struct MyTaggedStruct { a: tag(1) int32 }"
        ),
        (
            exception_as_key_invalid,
            "exception",
            "MyException",
            "exception MyException { }"
        ),
        (trait_as_key_invalid, "trait", "MyTrait", "trait MyTrait")
    );

    /// Valid dictionary key types test.
    macro_rules! test_simple_dictionary_key_types {
        ($(($test_name:ident, $key_type:expr)),*) => {
            $(
                #[test]
                fn $test_name() {
                    // Arrange
                    let slice = format!(
                        "
                        encoding = 2;
                        module Test;
                        typealias MyDict = dictionary<{key_type}, int32>;
                        ",
                        key_type=$key_type
                    );

                    // Act
                    let error_reporter = parse_for_errors(&slice);

                    // Assert
                    assert!(!error_reporter.has_errors(true));
                }
            )*
        }
    }

    // Valid dictionary key types test cases.
    test_simple_dictionary_key_types!(
        (uint8_as_key_valid, "uint8"),
        (uint16_as_key_valid, "uint16"),
        (uint32_as_key_valid, "uint32"),
        (uint64_as_key_valid, "uint64"),
        (int8_as_key_valid, "int8"),
        (int16_as_key_valid, "int16"),
        (int32_as_key_valid, "int32"),
        (int64_as_key_valid, "int64"),
        (varint32_as_key_valid, "varint32"),
        (varuint32_as_key_valid, "varuint32"),
        (varint62_as_key_valid, "varint62"),
        (varuint62_as_key_valid, "varuint62"),
        (string_as_key_valid, "string"),
        (bool_as_key_valid, "bool")
    );

    /// Compact structs containing allowed types as a key test.
    #[test]
    fn compact_struct_containing_allowed_primitives_as_key_valid() {
        // Arrange
        let slice = "
            encoding = 2;
            module Test;
            compact struct S {
                a : uint8,
                b : uint16,
                c : uint32,
                d : uint64,
                e : int8,
                f : int16,
                g : int32,
                h : int64,
                i : varint32,
                j : varuint32,
                k : varint62,
                l : varuint62,
                m : string,
                n : bool,
            }
            typealias MyDict = dictionary<S, int32>;
            ";

        // Act
        let error_reporter = parse_for_errors(&slice);

        // Assert
        assert!(!error_reporter.has_errors(true));
    }
}
