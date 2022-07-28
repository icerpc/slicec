// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_errors};
use crate::{assert_errors, assert_errors_new};
use slice::errors::{ErrorKind, RuleKind};
use slice::grammar::*;
use test_case::test_case;

#[test]
fn enumerator_default_values() {
    // Arrange
    let slice = "
        module Test;
        enum E {
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
            enum E {
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
fn enumerator_values_can_be_out_of_order() {
    // Arrange
    let slice = "
            module Test;
            enum E {
                A = 2,
                B = 1,
            }
        ";

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors!(error_reporter);
}

#[test]
fn validate_backing_type_out_of_bounds() {
    // Arranges
    let out_of_bounds_value = i16::MAX as i32 + 1;
    let slice = format!(
        "
            module Test;
            enum E: int16 {{
                A = {out_of_bounds_value},
            }}
        ",
        out_of_bounds_value = out_of_bounds_value,
    );
    let expected: ErrorKind = RuleKind::MustBeBounded(out_of_bounds_value as i64, -32768_i64, 32767_i64).into();

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors_new!(error_reporter, [&expected]);
}

#[test]
fn validate_backing_type_bounds() {
    // Arranges
    let bounds = (i16::MIN, i16::MAX);
    let slice = format!(
        "
            module Test;
            enum E: int16 {{
                A = {min},
                B = {max},
            }}
        ",
        min = bounds.0,
        max = bounds.1,
    );

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors!(error_reporter);
}

#[test_case("string"; "string")]
#[test_case("float32"; "float32")]
#[test_case("float64"; "float64")]
fn invalid_underlying_type(underlying_type: &str) {
    // Arrange
    let slice = format!(
        "
            module Test;
            enum E: {} {{
                A
            }}
        ",
        underlying_type,
    );
    let expected: ErrorKind = RuleKind::UnderlyingTypeMustBeIntegral(underlying_type.to_owned()).into();

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors_new!(error_reporter, [&expected]);
}

#[test_case("10"; "numeric identifier")]
#[test_case("😊"; "unicode identifier")]
#[ignore = "reason: validation not implemented"] // TODO
fn enumerator_invalid_identifiers(identifier: &str) {
    // Arrange
    let slice = format!(
        "
            module Test;
            enum E {{
                {identifier},
            }}
        ",
        identifier = identifier,
    );

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors!(error_reporter, [""]);
}

#[test]
fn optional_underlying_types_fail() {
    // Arrange
    let slice = "
        module Test;
        enum E: int32? { A = 1 }
    ";
    let expected: ErrorKind = RuleKind::CannotHaveOptionalUnderlyingType.into();

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors_new!(error_reporter, [&expected]);
}

#[test]
fn enumerators_must_be_unique() {
    // Arrange
    let slice = "
        module Test;
        enum E {
            A = 1,
            B = 1,
        }
    ";
    let expected = [
        RuleKind::MustBeUnique.into(),
        ErrorKind::new("The enumerator `A` has previous used the value `1`".to_owned()),
    ];

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors_new!(error_reporter, expected);
}

#[test]
fn automatically_assigned_values_will_not_overflow() {
    let slice = format!(
        "
            module Test;
            enum E {{
                A = {max_value},
                B,
            }}
        ",
        max_value = i64::MAX,
    );

    let error_reporter = parse_for_errors(&slice);

    assert_errors!(error_reporter, [
        " --> 5:17\n  |\n5 |                 B,␊\n  |                 ^\n  |\n  = Enumerator value out of range: B"
    ]);
}

#[test_case("unchecked enum", true ; "unchecked")]
#[test_case("enum", false ; "checked")]
fn can_be_unchecked(enum_definition: &str, expected_result: bool) {
    let slice = format!(
        "
            module Test;
            {enum_definition} E {{
                A,
                B,
            }}
        ",
        enum_definition = enum_definition,
    );

    let ast = parse_for_ast(slice);

    let enum_def = ast.find_element::<Enum>("Test::E").unwrap();
    assert_eq!(enum_def.is_unchecked, expected_result);
}

#[test]
fn checked_enums_can_not_be_empty() {
    let slice = "
        module Test;
        enum E {}
    ";
    let expected: ErrorKind = RuleKind::MustContainAtLeastOneValue.into();

    let error_reporter = parse_for_errors(slice);

    assert_errors_new!(error_reporter, [&expected]);
}

#[test]
fn unchecked_enums_can_be_empty() {
    let slice = "
        module Test;
        unchecked enum E {}
    ";

    let ast = parse_for_ast(slice);

    let enum_def = ast.find_element::<Enum>("Test::E").unwrap();
    assert_eq!(enum_def.enumerators.len(), 0);
}

mod slice1 {

    use crate::assert_errors_new;
    use crate::helpers::parsing_helpers::*;
    use slice::errors::{ErrorKind, RuleKind};

    #[test]
    fn enumerators_cannot_contain_negative_values() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            enum E {
                A = -1,
                B = -2,
                C = -3,
            }
        ";
        let expected_errors: [ErrorKind; 3] = [
            RuleKind::MustBePositive("enumerator values".to_owned()).into(),
            RuleKind::MustBePositive("enumerator values".to_owned()).into(),
            RuleKind::MustBePositive("enumerator values".to_owned()).into(),
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors_new!(error_reporter, expected_errors);
    }

    #[test]
    fn enumerators_cannot_contain_out_of_bounds_values() {
        // Arrange
        let slice = format!(
            "
                encoding = 1;
                module Test;
                enum E {{
                    A = {value},
                }}
            ",
            value = i32::MAX as i64 + 1
        );
        let expected: ErrorKind = RuleKind::MustBeBounded(i32::MAX as i64 + 1, 0_i64, i32::MAX as i64).into();

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors_new!(error_reporter, [&expected]);
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
            enum E: int32 {
                A = -1,
                B = -2,
                C = -3,
            }
        ";

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter);
    }

    #[test]
    fn enumerators_can_contain_values() {
        // Arrange
        let slice = "
            module Test;
            enum E: int16 {
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
