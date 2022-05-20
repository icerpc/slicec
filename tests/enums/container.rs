// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_errors};
use slice::grammar::*;
use slice::parse_from_string;
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
    let enum_ptr = ast.find_typed_type::<Enum>("Test::E").unwrap();
    let enum_def = enum_ptr.borrow();
    let enumerators = enum_def.enumerators();

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
    let enum_ptr = ast.find_typed_type::<Enum>("Test::E").unwrap();
    let enum_def = enum_ptr.borrow();
    let enumerators = enum_def.enumerators();

    assert_eq!(enumerators[1].value, 3);
    assert_eq!(enumerators[2].value, 4);
}

#[test]
fn out_of_order_enumerators_are_rejected() {
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
fn validate_backing_type_bounds() {
    // Arranges
    let out_of_bounds_value = i16::MAX as i32 + 1;
    let slice = format!(
        "
        module Test;
        enum E: int16 {{
            A = {out_of_bounds_value},
        }}
        ",
        out_of_bounds_value = out_of_bounds_value
    );

    // Act
    let error_reporter = parse_for_errors(&slice);

    // Assert
    assert_errors!(error_reporter,[
        "enumerator value '32768' is out of bounds. The value must be between `-32768..32767`, inclusive, for the underlying type `int16`"
    ]);
}

#[test_case("string")]
#[test_case("float32")]
#[test_case("float64")]
fn invalid_underlying_type(underlying_type: &str) {
    // Arrange
    let slice = format!(
        "
        module Test;
        enum E: {} {{
            A
        }}
        ",
        underlying_type
    );

    // Act
    let error_reporter = parse_for_errors(&slice);

    // Assert
    assert_errors!(error_reporter, [format!(
        "underlying type '{}' is not allowed for enums",
        underlying_type
    )]);
}

#[test_case("10")]
#[test_case("😊")]
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
        identifier = identifier
    );

    // Act
    let error_reporter = parse_for_errors(&slice);

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

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors!(error_reporter, [
        "underlying type 'int32' cannot be optional: enums cannot have optional underlying types"
    ]);
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

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors!(error_reporter, [
        "invalid enumerator value on enumerator `B`: enumerators must be unique",
        "The enumerator `A` has previous used the value `1`"
    ]);
}

#[test]
fn automatically_assigned_values_will_not_overflow() {
    let slice = format!(
        "module Test;
        enum E {{
            A = {max_value},
            B,
        }}",
        max_value = i64::MAX
    );

    let error = parse_from_string(&slice).err().unwrap();

    assert!(error.message.ends_with("Enumerator value out of range: B"));
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
        enum_definition = enum_definition
    );

    let ast = parse_for_ast(&slice);

    let enum_ptr = ast.find_typed_type::<Enum>("Test::E").unwrap();
    let enum_def = enum_ptr.borrow();

    assert_eq!(enum_def.is_unchecked, expected_result);
}

#[test]
fn checked_enums_can_not_be_empty() {
    let slice = "
        module Test;
        enum E {}
        ";

    let error_reporter = parse_for_errors(slice);

    assert_errors!(error_reporter, &[
        "enums must contain at least one enumerator"
    ]);
}

#[test]
fn unchecked_enums_can_be_empty() {
    let slice = "
        module Test;
        unchecked enum E {}
        ";

    let ast = parse_for_ast(slice);

    let enum_ptr = ast.find_typed_type::<Enum>("Test::E").unwrap();
    let enum_def = enum_ptr.borrow();

    assert_eq!(enum_def.enumerators.len(), 0);
}

mod slice1 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::*;

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
        let expected_errors = &[
            "invalid enumerator value on enumerator `A`: enumerators must be non-negative",
            "invalid enumerator value on enumerator `B`: enumerators must be non-negative",
            "invalid enumerator value on enumerator `C`: enumerators must be non-negative",
        ];

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, expected_errors);
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
        let expected_errors =
            &["invalid enumerator value on enumerator `A`: must be smaller than than 2147483647"];

        // Act
        let error_reporter = parse_for_errors(&slice);

        // Assert
        assert_errors!(error_reporter, expected_errors);
    }
}

mod slice2 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::*;
    use slice::grammar::*;

    ///
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
        let enum_ptr = ast.find_typed_type::<Enum>("Test::E").unwrap();
        let enum_def = enum_ptr.borrow();
        let enumerators = enum_def.enumerators();

        assert_eq!(enumerators.len(), 3);

        assert_eq!(enumerators[0].identifier(), "A");
        assert_eq!(enumerators[1].identifier(), "B");
        assert_eq!(enumerators[2].identifier(), "C");

        assert_eq!(enumerators[0].value, 1);
        assert_eq!(enumerators[1].value, 2);
        assert_eq!(enumerators[2].value, 3);

        assert!(matches!(
            *enum_def.underlying_type(Encoding::Slice2),
            Primitive::Int16
        ));
    }
}
