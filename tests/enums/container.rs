// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_diagnostics};
use slice::diagnostics::{Error, ErrorKind};
use slice::grammar::*;
use test_case::test_case;

#[test]
fn enumerator_default_values() {
    // Arrange
    let slice = "
        module Test;
        enum E
        {
            A,
            B,
            C,
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let enumerators = ast.find_element::<Enum>("Test::E").unwrap().enumerators();
    assert_eq!(enumerators[0].value, 0);
    assert_eq!(enumerators[1].value, 1);
    assert_eq!(enumerators[2].value, 2);
}

#[test]
fn subsequent_unsigned_value_is_incremented_previous_value() {
    // Arrange
    let slice = "
            module Test;
            enum E
            {
                A = 2,
                B,
                C,
            }
        ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let enumerators = ast.find_element::<Enum>("Test::E").unwrap().enumerators();
    assert_eq!(enumerators[1].value, 3);
    assert_eq!(enumerators[2].value, 4);
}

#[test]
fn implicit_enumerator_values_overflow_cleanly() {
    // Arrange
    let slice = "
        module Test;
        enum E
        {
            A,
            B = 170141183460469231731687303715884105727, // i128::MAX
            C,
        }
    ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = [
        Error::new(ErrorKind::EnumeratorValueOutOfBounds(
            "B".to_owned(),
            i128::MAX,
            -2147483648,
            2147483647,
        )),
        Error::new(ErrorKind::EnumeratorValueOutOfBounds(
            "C".to_owned(),
            i128::MIN,
            -2147483648,
            2147483647,
        )),
    ];
    assert_errors!(diagnostic_reporter, expected);
}

#[test]
fn enumerator_values_can_be_out_of_order() {
    // Arrange
    let slice = "
            module Test;
            enum E
            {
                A = 2,
                B = 1,
            }
        ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(diagnostic_reporter);
}

#[test]
fn validate_backing_type_out_of_bounds() {
    // Arranges
    let out_of_bounds_value = i16::MAX as i128 + 1;
    let slice = format!(
        "
            module Test;
            enum E: int16
            {{
                A = {out_of_bounds_value},
            }}
        "
    );

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::EnumeratorValueOutOfBounds(
        "A".to_owned(),
        out_of_bounds_value,
        -32768_i128,
        32767_i128,
    ));
    assert_errors!(diagnostic_reporter, [&expected]);
}

#[test]
fn validate_backing_type_bounds() {
    // Arranges
    let min = i16::MIN;
    let max = i16::MAX;
    let slice = format!(
        "
            module Test;
            enum E: int16
            {{
                A = {min},
                B = {max},
            }}
        "
    );

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(diagnostic_reporter);
}

#[test_case("string"; "string")]
#[test_case("float32"; "float32")]
#[test_case("float64"; "float64")]
fn invalid_underlying_type(underlying_type: &str) {
    // Arrange
    let slice = format!(
        "
            module Test;
            enum E: {underlying_type}
            {{
                A
            }}
        "
    );

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::UnderlyingTypeMustBeIntegral(
        "E".to_owned(),
        underlying_type.to_owned(),
    ));
    assert_errors!(diagnostic_reporter, [&expected]);
}

#[test_case("10", "expected one of \"[\", \"}\", doc_comment, identifier, but found 'IntegerLiteral(\"10\")'"; "numeric identifier")]
#[test_case("😊", "unknown symbol '😊'"; "unicode identifier")]
fn enumerator_invalid_identifiers(identifier: &str, expected: &str) {
    // Arrange
    let slice = format!(
        "
            module Test;
            enum E
            {{
                {identifier},
            }}
        "
    );

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    assert_errors!(diagnostic_reporter, [expected]);
}

#[test]
fn optional_underlying_types_fail() {
    // Arrange
    let slice = "
        module Test;

        enum E: int32?
        {
            A = 1
        }
    ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::CannotUseOptionalUnderlyingType("E".to_owned()));
    assert_errors!(diagnostic_reporter, [&expected]);
}

#[test]
fn enumerators_must_be_unique() {
    // Arrange
    let slice = "
        module Test;

        enum E
        {
            A = 1,
            B = 1,
        }
    ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected =
        Error::new(ErrorKind::DuplicateEnumeratorValue(1)).add_note("the value was previously used by `A` here:", None);
    assert_errors!(diagnostic_reporter, [&expected]);
}

#[test_case("unchecked enum", true ; "unchecked")]
#[test_case("enum", false ; "checked")]
fn can_be_unchecked(enum_definition: &str, expected_result: bool) {
    // Arrange
    let slice = format!(
        "
            module Test;
            {enum_definition} E
            {{
                A,
                B,
            }}
        "
    );

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let enum_def = ast.find_element::<Enum>("Test::E").unwrap();
    assert_eq!(enum_def.is_unchecked, expected_result);
}

#[test]
fn checked_enums_can_not_be_empty() {
    let slice = "
        module Test;

        enum E
        {
        }
    ";
    let expected = Error::new(ErrorKind::MustContainEnumerators("E".to_owned()));

    let diagnostic_reporter = parse_for_diagnostics(slice);

    assert_errors!(diagnostic_reporter, [&expected]);
}

#[test]
fn unchecked_enums_can_be_empty() {
    // Arrange
    let slice = "
        module Test;

        unchecked enum E
        {
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let enum_def = ast.find_element::<Enum>("Test::E").unwrap();
    assert_eq!(enum_def.enumerators.len(), 0);
}

#[test]
fn enumerators_support_different_base_literals() {
    // Arrange
    let slice = "
        module Test;

        enum E
        {
            B = 0b1001111,
            D = 128,
            H = 0xA4FD,
            N = -0xbc81,
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert_eq!(ast.find_element::<Enumerator>("Test::E::B").unwrap().value, 0b1001111);
    assert_eq!(ast.find_element::<Enumerator>("Test::E::D").unwrap().value, 128);
    assert_eq!(ast.find_element::<Enumerator>("Test::E::H").unwrap().value, 0xA4FD);
    assert_eq!(ast.find_element::<Enumerator>("Test::E::N").unwrap().value, -0xbc81);
}

#[test]
fn duplicate_enumerators_are_disallowed_across_different_bases() {
    // Arrange
    let slice = "
        module Test;

        enum E
        {
            B = 0b1001111,
            D = 79,
        }
    ";

    // Act
    let diagnostic_reporter = parse_for_diagnostics(slice);

    // Assert
    let expected = Error::new(ErrorKind::DuplicateEnumeratorValue(79));
    assert_errors!(diagnostic_reporter, [&expected]);
}

mod slice1 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::*;
    use slice::diagnostics::{Error, ErrorKind};

    #[test]
    fn enumerators_cannot_contain_negative_values() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;

            enum E
            {
                A = -1,
                B = -2,
                C = -3,
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        const MAX_VALUE: i128 = i32::MAX as i128;
        let expected_errors: [Error; 3] = [
            Error::new(ErrorKind::EnumeratorValueOutOfBounds("A".to_owned(), -1, 0, MAX_VALUE)),
            Error::new(ErrorKind::EnumeratorValueOutOfBounds("B".to_owned(), -2, 0, MAX_VALUE)),
            Error::new(ErrorKind::EnumeratorValueOutOfBounds("C".to_owned(), -3, 0, MAX_VALUE)),
        ];
        assert_errors!(diagnostic_reporter, expected_errors);
    }

    #[test]
    fn enumerators_cannot_contain_out_of_bounds_values() {
        // Arrange
        let value = i32::MAX as i128 + 1;
        let slice = format!(
            "
                encoding = 1;
                module Test;

                enum E
                {{
                    A = {value},
                }}
            "
        );

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::EnumeratorValueOutOfBounds(
            "A".to_owned(),
            value,
            0,
            i32::MAX as i128,
        ));
        assert_errors!(diagnostic_reporter, [&expected]);
    }
}

mod slice2 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::*;
    use slice::grammar::*;

    #[test]
    fn enumerators_can_contain_negative_values() {
        // Arrange
        let slice = "
            module Test;

            enum E: int32
            {
                A = -1,
                B = -2,
                C = -3,
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(diagnostic_reporter);
    }

    #[test]
    fn enumerators_can_contain_values() {
        // Arrange
        let slice = "
            module Test;

            enum E: int16
            {
                A = 1,
                B = 2,
                C = 3,
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let enum_def = ast.find_element::<Enum>("Test::E").unwrap();
        let enumerators = enum_def.enumerators();

        assert_eq!(enumerators.len(), 3);
        assert_eq!(enumerators[0].identifier(), "A");
        assert_eq!(enumerators[1].identifier(), "B");
        assert_eq!(enumerators[2].identifier(), "C");
        assert_eq!(enumerators[0].value, 1);
        assert_eq!(enumerators[1].value, 2);
        assert_eq!(enumerators[2].value, 3);
        assert!(matches!(
            enum_def.underlying.as_ref().unwrap().definition(),
            Primitive::Int16,
        ));
    }
}
