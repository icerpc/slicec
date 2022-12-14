// Copyright (c) ZeroC, Inc. All rights reserved.

mod slice1 {

    use slice::diagnostics::{Error, ErrorKind};
    use slice::grammar::Encoding;

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_diagnostics;

    /// Verifies that the slice parser with the Slice1 encoding emits errors when parsing an enum
    /// that has an underlying type.
    #[test]
    fn underlying_types_fail() {
        // Arrange
        let slice = "
            encoding = 1;
            module Test;

            unchecked enum E : int32
            {
            }
        ";

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        let expected = Error::new(ErrorKind::NotSupportedWithEncoding {
            kind: "enum".to_owned(),
            identifier: "E".to_owned(),
            encoding: Encoding::Slice1,
        })
        .add_note("file encoding was set to Slice1 here:", None)
        .add_note(
            "enums with underlying types are not supported by the Slice1 encoding",
            None,
        );

        assert_errors!(diagnostic_reporter, [&expected]);
    }
}

mod slice2 {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::parse_for_diagnostics;
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

                unchecked enum E : {valid_type}
                {{
                }}
            "
        );

        // Act
        let diagnostic_reporter = parse_for_diagnostics(slice);

        // Assert
        assert_errors!(diagnostic_reporter);
    }
}
