// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::assert_errors;
use crate::helpers::parsing_helpers::*;

mod slice1 {

    use super::*;

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

    use super::*;
    use slice::grammar::*;

    ///
    #[test]
    fn enums_can_be_empty() {
        // Arrange
        let slice = "
        encoding = 2;
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
        encoding = 2;
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
        encoding = 2;
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
