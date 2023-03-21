// Copyright (c) ZeroC, Inc.

use crate::test_helpers::*;
use slice::diagnostics::{Diagnostic, Error};
use slice::grammar::*;
use test_case::test_case;

#[test]
fn enumerator_default_values() {
    // Arrange
    let slice = "
        module Test
        enum E: uint8 {
            A
            B
            C
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let enumerators = ast.find_element::<Enum>("Test::E").unwrap().enumerators();
    assert_eq!(enumerators[0].value(), 0);
    assert_eq!(enumerators[1].value(), 1);
    assert_eq!(enumerators[2].value(), 2);
}

#[test]
fn subsequent_unsigned_value_is_incremented_previous_value() {
    // Arrange
    let slice = "
        module Test
        enum E: uint8 {
            A = 2
            B
            C
        }
        ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    let enumerators = ast.find_element::<Enum>("Test::E").unwrap().enumerators();
    assert_eq!(enumerators[1].value(), 3);
    assert_eq!(enumerators[2].value(), 4);
}

#[test]
fn implicit_enumerator_values_overflow_cleanly() {
    // Arrange
    let slice = "
        module Test
        enum E: varint32 {
            A
            B = 170141183460469231731687303715884105727 // i128::MAX
            C
        }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = [
        Diagnostic::new(Error::EnumeratorValueOutOfBounds {
            enumerator_identifier: "B".to_owned(),
            value: i128::MAX,
            min: -2147483648,
            max: 2147483647,
        }),
        Diagnostic::new(Error::EnumeratorValueOutOfBounds {
            enumerator_identifier: "C".to_owned(),
            value: i128::MIN,
            min: -2147483648,
            max: 2147483647,
        }),
    ];
    check_diagnostics(diagnostics, expected);
}

#[test]
fn enumerator_values_can_be_out_of_order() {
    // Arrange
    let slice = "
            module Test
            enum E: uint8 {
                A = 2
                B = 1
            }
        ";

    // Act/Assert
    assert_parses(slice);
}

#[test]
fn validate_backing_type_out_of_bounds() {
    // Arranges
    let out_of_bounds_value = i16::MAX as i128 + 1;
    let slice = format!(
        "
            module Test
            enum E: int16 {{
                A = {out_of_bounds_value}
            }}
        "
    );

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::EnumeratorValueOutOfBounds {
        enumerator_identifier: "A".to_owned(),
        value: out_of_bounds_value,
        min: -32768_i128,
        max: 32767_i128,
    });
    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn validate_backing_type_bounds() {
    // Arranges
    let min = i16::MIN;
    let max = i16::MAX;
    let slice = format!(
        "
            module Test
            enum E: int16 {{
                A = {min}
                B = {max}
            }}
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
            enum E: {underlying_type} {{
                A
            }}
        "
    );

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::UnderlyingTypeMustBeIntegral {
        enum_identifier: "E".to_owned(),
        kind: underlying_type.to_owned(),
    });
    check_diagnostics(diagnostics, [expected]);
}

#[test_case("10", "expected one of '[', '}', 'doc comment', or 'identifier', but found '10'"; "numeric identifier")]
#[test_case("😊", "unknown symbol '😊'"; "unicode identifier")]
fn enumerator_invalid_identifiers(identifier: &str, expected_message: &str) {
    // Arrange
    let slice = format!(
        "
            module Test
            enum E: uint8 {{
                {identifier}
            }}
        "
    );

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::Syntax {
        message: expected_message.to_owned(),
    });
    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn optional_underlying_types_fail() {
    // Arrange
    let slice = "
        module Test

        enum E: int32? {
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

#[test]
fn enumerators_must_be_unique() {
    // Arrange
    let slice = "
        module Test

        enum E: uint8 {
            A = 1
            B = 1
        }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::DuplicateEnumeratorValue { enumerator_value: 1 })
        .add_note("the value was previously used by 'A' here:", None);

    check_diagnostics(diagnostics, [expected]);
}

#[test_case("unchecked enum", true ; "unchecked")]
#[test_case("enum", false ; "checked")]
fn can_be_unchecked(enum_definition: &str, expected_result: bool) {
    // Arrange
    let slice = format!(
        "
            module Test
            {enum_definition} E: uint8 {{
                A
                B
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
    // Arrange
    let slice = "
        module Test

        enum E: uint8 {}
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::MustContainEnumerators {
        enum_identifier: "E".to_owned(),
    });
    check_diagnostics(diagnostics, [expected]);
}

#[test]
fn unchecked_enums_can_be_empty() {
    // Arrange
    let slice = "
        module Test

        unchecked enum E: uint8 {}
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
        module Test

        enum E: varint32 {
            B = 0b1001111
            D = 128
            H = 0xA4FD
            N = -0xbc81
        }
    ";

    // Act
    let ast = parse_for_ast(slice);

    // Assert
    assert_eq!(ast.find_element::<Enumerator>("Test::E::B").unwrap().value(), 0b1001111);
    assert_eq!(ast.find_element::<Enumerator>("Test::E::D").unwrap().value(), 128);
    assert_eq!(ast.find_element::<Enumerator>("Test::E::H").unwrap().value(), 0xA4FD);
    assert_eq!(ast.find_element::<Enumerator>("Test::E::N").unwrap().value(), -0xbc81);
}

#[test]
fn duplicate_enumerators_are_disallowed_across_different_bases() {
    // Arrange
    let slice = "
        module Test

        enum E: uint16 {
            B = 0b1001111
            D = 79
        }
    ";

    // Act
    let diagnostics = parse_for_diagnostics(slice);

    // Assert
    let expected = Diagnostic::new(Error::DuplicateEnumeratorValue { enumerator_value: 79 });
    check_diagnostics(diagnostics, [expected]);
}

mod slice1 {

    use crate::test_helpers::*;
    use slice::diagnostics::{Diagnostic, Error};

    #[test]
    fn enumerators_cannot_contain_negative_values() {
        // Arrange
        let slice = "
            encoding = 1
            module Test

            enum E {
                A = -1
                B = -2
                C = -3
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        const MAX_VALUE: i128 = i32::MAX as i128;
        let expected = [
            Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                enumerator_identifier: "A".to_owned(),
                value: -1,
                min: 0,
                max: MAX_VALUE,
            }),
            Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                enumerator_identifier: "B".to_owned(),
                value: -2,
                min: 0,
                max: MAX_VALUE,
            }),
            Diagnostic::new(Error::EnumeratorValueOutOfBounds {
                enumerator_identifier: "C".to_owned(),
                value: -3,
                min: 0,
                max: MAX_VALUE,
            }),
        ];
        check_diagnostics(diagnostics, expected);
    }

    #[test]
    fn enumerators_cannot_contain_out_of_bounds_values() {
        // Arrange
        let value = i32::MAX as i128 + 1;
        let slice = format!(
            "
                encoding = 1
                module Test

                enum E {{
                    A = {value}
                }}
            "
        );

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::EnumeratorValueOutOfBounds {
            enumerator_identifier: "A".to_owned(),
            value,
            min: 0,
            max: i32::MAX as i128,
        });
        check_diagnostics(diagnostics, [expected]);
    }
}

mod slice2 {

    use crate::test_helpers::*;
    use slice::grammar::*;

    #[test]
    fn enumerators_can_contain_negative_values() {
        // Arrange
        let slice = "
            module Test

            enum E: int32 {
                A = -1
                B = -2
                C = -3
            }
        ";

        // Act/Assert
        assert_parses(slice);
    }

    #[test]
    fn enumerators_can_contain_values() {
        // Arrange
        let slice = "
            module Test

            enum E: int16 {
                A = 1
                B = 2
                C = 3
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
        assert_eq!(enumerators[0].value(), 1);
        assert_eq!(enumerators[1].value(), 2);
        assert_eq!(enumerators[2].value(), 3);
        assert!(matches!(
            enum_def.underlying.as_ref().unwrap().definition(),
            Primitive::Int16,
        ));
    }

    #[test]
    fn explicit_enumerator_value_kinds() {
        let slice = "
        module Test

        enum A: uint8 {
            u = 1
            v = 2
            w = 3
        }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let enum_def_a = ast.find_element::<Enum>("Test::A").unwrap();
        let enumerators_a = enum_def_a.enumerators();

        assert!(matches!(enumerators_a[0].value, EnumeratorValue::Explicit(..)));
        assert!(matches!(enumerators_a[1].value, EnumeratorValue::Explicit(..)));
        assert!(matches!(enumerators_a[2].value, EnumeratorValue::Explicit(..)));
    }
}
