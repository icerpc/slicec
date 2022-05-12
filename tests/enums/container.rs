// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_errors};
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
    let enum_ptr = ast.find_typed_type::<Enum>("Test::E").unwrap();
    let enum_def = enum_ptr.borrow();
    let enumerators = enum_def.enumerators();

    assert_eq!(enumerators[0].value, 0);
    assert_eq!(enumerators[1].value, 1);
    assert_eq!(enumerators[2].value, 2);
}

#[test]
fn should_be_monotonic_increasing() {
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
#[ignore = "reason: validation not implemented"] // TODO
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
    assert_errors!(error_reporter, [""]);
}

#[test]
#[ignore = "reason: validation not implemented"] // TODO
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
    assert_errors!(error_reporter, [""]);
}

#[test_case("string")]
#[test_case("float32")]
#[test_case("float64")]
#[ignore = "reason: validation not implemented"] // TODO
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
    assert_errors!(error_reporter, [""]);
}

#[test_case("10")]
#[test_case("ábč")]
#[test_case("true")]
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
#[ignore = "reason: validation not implemented"] // TODO
fn optional_underlying_types_fail() {
    // Arrange
    let slice = "
        module Test;
        enum E: int32? { A = 1 }
    ";

    // Act
    let error_reporter = parse_for_errors(slice);

    // Assert
    assert_errors!(error_reporter, [""]);
}

#[test]
#[ignore = "reason: validation not implemented"] // TODO
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
    assert_errors!(error_reporter, [""]);
}

mod slice1 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::*;

    /// * Note this passes and should not. Austin suspects similar to variable size backing types
    /// working. This is most likely a result of not producing the correct errors.
    #[test]
    #[ignore]
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
        let expected_errors = &["ERROR"]; // TODO: Add the relevant error message once fixed

        // Act
        let error_reporter = parse_for_errors(slice);

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
    fn enums_can_be_empty() {
        // Arrange
        let slice = "
        module Test;
        enum E {}
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let enum_ptr = ast.find_typed_type::<Enum>("Test::E").unwrap();
        let enum_def = enum_ptr.borrow();
        let enumerators = enum_def.enumerators();
        assert_eq!(enumerators.len(), 0);
    }

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

        assert!(matches!(*enum_def.underlying_type(), Primitive::Int16));
    }
}
