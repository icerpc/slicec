// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::parse_for_errors;

mod slice1 {}

mod slice2 {

    use super::*;
    use test_case::test_case;

    /// Invalid primitive dictionary key types test.
    #[test_case("int32?")]
    #[test_case("proxy")]
    #[test_case("AnyClass")]
    #[ignore] // Remove ignore when disallowed key errors are added.
    fn invalid_simple_type_dictionary_key_types_fails(key_type: &str) {
        // Arrange
        let slice = &format!(
            "
            module Test;
            typealias TestDict = dictionary<{key_type}, int32>;
            ",
            key_type = key_type
        );
        let expected_errors = &[
            &format!("{} cannot be used as a dictionary key type", key_type,),
            "'TestDict' was defined here:",
        ]; // Add the expected errors here when disallowed key errors are added.

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, expected_errors);
    }

    /// Invalid Constructed dictionary key types test.
    #[test_case("dictionary", "MyDict", "typealias MyDict = dictionary<int32, int32>" ; "dictionary as key type")]
    #[test_case("proxy", "I", "interface I {}" ; "proxy as key type")]
    #[test_case("struct", "MyStruct", "struct MyStruct { }"; "struct as key type")]
    #[test_case(
        "struct",
        "MyTaggedStruct",
        "struct MyTaggedStruct { a: tag(1) int32 }"
        ; "struct with tag as key type"
    )]
    #[test_case("exception", "MyException", "exception MyException { }"; "exception as key type")]
    #[test_case("trait", "MyTrait", "trait MyTrait"; "trait as key type")]
    #[test_case("custom", "MyCustom", "custom MyCustom"; "custom as key type")]
    #[ignore] // Remove ignore when disallowed key errors are added.
    fn constructed_dictionary_key_types_fails(key_type: &str, key_ident: &str, key_def: &str) {
        // Arrange
        let slice = &format!(
            "
            module test;

            {key_def}

            typealias TestDict = dictionary<{key_type}, int32>;
            ",
            key_def = key_def,
            key_type = key_ident,
        );

        let expected_errors: [&str; 2] = [
            &format!(
                "{} '{}' cannot be used as a dictionary key type",
                key_type, key_ident,
            ),
            &format!("{} was defined here:", key_ident,),
        ];
        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, &expected_errors);
    }

    // Valid dictionary key types
    #[test_case("uint8")]
    #[test_case("uint16")]
    #[test_case("uint32")]
    #[test_case("uint64")]
    #[test_case("int8")]
    #[test_case("int16")]
    #[test_case("int32")]
    #[test_case("int64")]
    #[test_case("varint32")]
    #[test_case("varuint32")]
    #[test_case("varint62")]
    #[test_case("varuint62")]
    #[test_case("string")]
    #[test_case("bool")]
    fn simple_dictionary_key_types(key_type: &str) {
        // Arrange
        let slice = format!(
            "
            module Test;
            typealias MyDict = dictionary<{key_type}, int32>;
            ",
            key_type = key_type
        );

        // Act
        let error_reporter = parse_for_errors(&slice);

        // Assert
        assert_errors!(error_reporter);
    }
}
