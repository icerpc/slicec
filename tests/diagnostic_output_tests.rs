// Copyright (c) ZeroC, Inc. All rights reserved.

mod output {

    use slice::command_line::{DiagnosticFormat, SliceOptions};
    use slice::parse_from_strings;

    fn default_options() -> SliceOptions {
        SliceOptions {
            sources: vec![],
            references: vec![],
            warn_as_error: true,
            disable_color: false,
            diagnostic_format: DiagnosticFormat::Human,
            validate: false,
            output_dir: None,
        }
    }

    #[test]
    fn output_to_json() {
        let slice = r#"
        module  Foo;

        interface I {
            /// @param x this is an x
            op();
        }

        enum E {}
        "#;

        // Set the output format to JSON.
        let mut default_options = default_options();
        default_options.diagnostic_format = DiagnosticFormat::Json;

        // Parse the Slice file.
        let parsed_data = parse_from_strings(&[slice], Some(default_options)).expect_err("Expected Slice errors");

        let mut output: Vec<u8> = Vec::new();

        // Act
        parsed_data.emit_diagnostics(&mut output);

        // Assert
        let expected = concat!(
            r#"{"message":"doc comment has a param tag for 'x', but there is no parameter by that name","severity":"warning","span":{"start":{"row":5,"col":13},"end":{"row":6,"col":13},"file":"string-0"},"notes":[],"error_code":"W001"}"#,
            "\n",
            r#"{"message":"invalid enum `E`: enums must contain at least one enumerator","severity":"error","span":{"start":{"row":9,"col":9},"end":{"row":9,"col":15},"file":"string-0"},"notes":[],"error_code":"E010"}"#,
            "\n",
        );
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn output_to_console() {
        let slice = r#"
        module  Foo;

        interface I {
            /// @param x this is an x
            op();
        }

        enum E {}
        "#;

        // Disable ANSI codes.
        let mut default_options = default_options();
        default_options.disable_color = true;

        // Parse the Slice file.
        let parsed_data = parse_from_strings(&[slice], Some(default_options)).expect_err("Expected Slice errors");

        let mut output: Vec<u8> = Vec::new();

        // Act
        parsed_data.emit_diagnostics(&mut output);

        // Assert
        let expected = "\
warning [W001]: doc comment has a param tag for 'x', but there is no parameter by that name
 --> string-0:5:13
    |
5   |             /// @param x this is an x
6   |             op();
    |             -------------------------
    |
error [E010]: invalid enum `E`: enums must contain at least one enumerator
 --> string-0:9:9
    |
9   |         enum E {}
    |         ------
    |
";

        assert_eq!(expected, String::from_utf8(output).unwrap());
    }
}
