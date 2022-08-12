// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice1 {

    use slice::diagnostics::{DiagnosticKind, LogicKind};
    use slice::grammar::Encoding;

    use crate::assert_errors_new;
    use crate::helpers::parsing_helpers::parse_for_errors;

    /// Verifies that the slice parser with the Slice1 encoding emits errors when parsing an enum
    /// that has an underlying type.
    #[test]
    fn underlying_types_fail() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;
            unchecked enum E : int32 {}
        ";

        // Act
        let diagnostic_reporter = parse_for_errors(slice);

        // Assert
        let expected_errors = [
            LogicKind::NotSupportedWithEncoding("enum".to_owned(), "E".to_owned(), Encoding::Slice1).into(),
            DiagnosticKind::new_note("file encoding was set to Slice1 here:".to_owned()),
            DiagnosticKind::new_note("enums with underlying types are not supported by the Slice1 encoding".to_owned()),
        ];
        assert_errors_new!(diagnostic_reporter, expected_errors);
    }
}

mod slice2 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_errors;
    use test_case::test_case;

    #[test_case("uint8"; "uint8")]
    #[test_case("int16"; "int16")]
    #[test_case("uint16"; "uint16")]
    #[test_case("int32"; "int32")]
    #[test_case("uint32"; "uint32")]
    #[test_case("varint32"; "varint32")]
    #[test_case("varuint32"; "varuint32")]
    #[test_case("varint62"; "varint62")]
    #[test_case("varuint62"; "varuint62")]
    fn supported_numeric_underlying_types_succeed(valid_type: &str) {
        // Arrange
        let slice = &format!(
            "
                module Test;
                unchecked enum E : {} {{}}
            ",
            valid_type,
        );

        // Act
        let diagnostic_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(diagnostic_reporter);
    }
}
