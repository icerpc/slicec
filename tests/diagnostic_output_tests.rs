// Copyright (c) ZeroC, Inc.

mod output {

    use slice::command_line::{DiagnosticFormat, SliceOptions};
    use slice::compile_from_strings;
    use slice::diagnostics::{Diagnostic, Error};
    use slice::slice_file::Span;

    #[test]
    fn output_to_json() {
        let slice = r#"
        module  Foo

        interface I {
            /// @param x: this is an x
            op()
        }

        enum E: int8 {}
        "#;

        // Set the output format to JSON.
        let options = SliceOptions {
            diagnostic_format: DiagnosticFormat::Json,
            ..Default::default()
        };

        // Parse the Slice file.
        let compilation_data = compile_from_strings(&[slice], Some(options)).expect("Expected errors");

        let mut output: Vec<u8> = Vec::new();

        // Act
        compilation_data.emit_diagnostics(&mut output);

        // Assert
        let expected = concat!(
            r#"{"message":"doc comment has a param tag for 'x', but there is no parameter by that name","severity":"warning","span":{"start":{"row":5,"col":17},"end":{"row":5,"col":39},"file":"string-0"},"notes":[],"error_code":"W003"}"#,
            "\n",
            r#"{"message":"invalid enum 'E': enums must contain at least one enumerator","severity":"error","span":{"start":{"row":9,"col":9},"end":{"row":9,"col":15},"file":"string-0"},"notes":[],"error_code":"E010"}"#,
            "\n",
        );
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn output_to_console() {
        let slice = r#"
        module  Foo

        interface I {
            /// @param x: this is an x
            op1()

            op2(tag(1)
    x:
                    int32, tag(2) y: bool?,
            )
        }

        enum E: int8 {}
        "#;

        // Disable ANSI codes.
        let options = SliceOptions {
            disable_color: true,
            ..Default::default()
        };

        // Parse the Slice file.
        let compilation_data = compile_from_strings(&[slice], Some(options)).expect("Expected errors");

        let mut output: Vec<u8> = Vec::new();

        // Act
        compilation_data.emit_diagnostics(&mut output);

        // Assert
        let expected = "\
warning [W003]: doc comment has a param tag for 'x', but there is no parameter by that name
 --> string-0:5:17
  |
5 |             /// @param x: this is an x
  |                 ----------------------
  |
error [E019]: invalid tag on member 'x': tagged members must be optional
 --> string-0:8:17
   |
8  |             op2(tag(1)
   |                 ------
9  |     x:
   | ------
10 |                     int32, tag(2) y: bool?,
   | -------------------------
   |
error [E010]: invalid enum 'E': enums must contain at least one enumerator
 --> string-0:14:9
   |
14 |         enum E: int8 {}
   |         ------
   |
";

        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn allow_warnings_flag_with_no_args() {
        let slice = r#"
        module  Foo

        interface I {
            /// @param x: this is an x
            op()
        }

        "#;
        // allow: Some([]),
        // Set the output format to JSON.
        let options = SliceOptions {
            diagnostic_format: DiagnosticFormat::Json,
            allow: Some(vec![]),
            ..Default::default()
        };

        // Parse the Slice file.
        let compilation_data = compile_from_strings(&[slice], Some(options)).expect("Expected errors");

        let mut output: Vec<u8> = Vec::new();

        // Act
        compilation_data.emit_diagnostics(&mut output);

        // Assert
        assert_eq!(String::new(), String::from_utf8(output).unwrap());
    }

    #[test]
    fn allow_warnings_flag_with_args() {
        let slice = r#"
        module  Foo

        interface I {
            /// @param x: this is an x
            /// @returns: this is a return
            op()
        }

        "#;
        // Set the output format to JSON.
        let options = SliceOptions {
            diagnostic_format: DiagnosticFormat::Json,
            allow: Some(vec!["W004".to_string()]),
            ..Default::default()
        };

        // Parse the Slice file.
        let compilation_data = compile_from_strings(&[slice], Some(options)).expect("Expected errors");

        let mut output: Vec<u8> = Vec::new();

        // Act
        compilation_data.emit_diagnostics(&mut output);

        // Assert
        // Only one of the two warnings should be allowed.
        let expected = concat!(
            r#"{"message":"doc comment has a param tag for 'x', but there is no parameter by that name","severity":"warning","span":{"start":{"row":5,"col":17},"end":{"row":5,"col":39},"file":"string-0"},"notes":[],"error_code":"W003"}"#,
            "\n",
        );
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn notes_with_same_span_as_diagnostic_suppressed() {
        // Arrange
        let slice = "\
            encoding = Slice2
            module Foo
        ";

        // Disable ANSI codes.
        let options = SliceOptions {
            disable_color: true,
            ..Default::default()
        };

        let mut compilation_data = compile_from_strings(&[slice], Some(options)).expect("Expected errors");
        let mut output: Vec<u8> = Vec::new();

        // Report a diagnostic with a note that has the same span as the diagnostic.
        let span = Span {
            start: (1, 1).into(),
            end: (2, 2).into(),
            file: "string-0".to_owned(),
        };

        Diagnostic::new(Error::Syntax {
            message: "foo".to_owned(),
        })
        .set_span(&span)
        .add_note("bar", Some(&span))
        .report(&mut compilation_data.diagnostic_reporter);

        // Act
        compilation_data.emit_diagnostics(&mut output);

        // Assert
        let expected = "\
error [E002]: foo
 --> string-0:1:1\n  |
1 | encoding = Slice2
  | -----------------
2 |             module Foo
  | -
  |
    = note: bar
";
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }
}
