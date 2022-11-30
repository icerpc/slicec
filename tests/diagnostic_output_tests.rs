// Copyright (c) ZeroC, Inc. All rights reserved.

mod output {

    use slice::command_line::{DiagnosticFormat, SliceOptions};
    use slice::compile_from_strings;

    #[test]
    fn output_to_json() {
        let slice = r#"
        module  Foo;

        interface I
        {
            /// @param x this is an x
            op();
        }

        enum E
        {
        }
        "#;

        // Set the output format to JSON.
        let mut default_options = SliceOptions::default();
        default_options.diagnostic_format = DiagnosticFormat::Json;

        // Parse the Slice file.
        let compilation_data = compile_from_strings(&[slice], Some(default_options)).expect("Expected errors");

        let mut output: Vec<u8> = Vec::new();

        // Act
        compilation_data.emit_diagnostics(&mut output);

        // Assert
        let expected = concat!(
            r#"{"message":"doc comment has a param tag for 'x', but there is no parameter by that name","severity":"warning","span":{"start":{"row":6,"col":13},"end":{"row":6,"col":38},"file":"string-0"},"notes":[],"error_code":"W001"}"#,
            "\n",
            r#"{"message":"invalid enum `E`: enums must contain at least one enumerator","severity":"error","span":{"start":{"row":10,"col":9},"end":{"row":10,"col":15},"file":"string-0"},"notes":[],"error_code":"E010"}"#,
            "\n",
        );
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn output_to_console() {
        let slice = r#"
        module  Foo;

        interface I
        {
            /// @param x this is an x
            op1();

            op2(x:
    tag(1)
                    int32, y: tag(2) bool?,
            );
        }

        enum E
        {
        }
        "#;

        // Disable ANSI codes.
        let mut default_options = SliceOptions::default();
        default_options.disable_color = true;

        // Parse the Slice file.
        let compilation_data = compile_from_strings(&[slice], Some(default_options)).expect("Expected errors");

        let mut output: Vec<u8> = Vec::new();

        // Act
        compilation_data.emit_diagnostics(&mut output);

        // Assert
        let expected = "\
warning [W001]: doc comment has a param tag for 'x', but there is no parameter by that name
 --> string-0:6:13
  |
6 |             /// @param x this is an x
  |             -------------------------
  |
error [E020]: invalid tag on member `x`: tagged members must be optional
 --> string-0:9:17
   |
9  |             op2(x:
   |                 --
10 |     tag(1)
   | ----------
11 |                     int32, y: tag(2) bool?,
   | -------------------------
   |
error [E010]: invalid enum `E`: enums must contain at least one enumerator
 --> string-0:15:9
   |
15 |         enum E
   |         ------
   |
";

        assert_eq!(expected, String::from_utf8(output).unwrap());
    }
}
