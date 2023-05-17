// Copyright (c) ZeroC, Inc.

mod output {

    use slice::compile_from_strings;
    use slice::diagnostics::{Diagnostic, Error};
    use slice::slice_file::Span;
    use slice::slice_options::{DiagnosticFormat, SliceOptions};

    #[test]
    fn output_to_json() {
        let slice = r#"
        module Foo

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
        let compilation_state = compile_from_strings(&[slice], Some(options), |_| {}, |_| {});

        let mut output: Vec<u8> = Vec::new();

        // Act
        compilation_state.emit_diagnostics(&mut output);

        // Assert
        let expected = concat!(
            r#"{"message":"doc comment has a param tag for 'x', but there is no parameter by that name","severity":"warning","span":{"start":{"row":5,"col":17},"end":{"row":5,"col":39},"file":"string-0"},"notes":[],"error_code":"IncorrectDocComment"}"#,
            "\n",
            r#"{"message":"invalid enum 'E': enums must contain at least one enumerator","severity":"error","span":{"start":{"row":9,"col":9},"end":{"row":9,"col":15},"file":"string-0"},"notes":[],"error_code":"E010"}"#,
            "\n",
        );
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn output_to_console() {
        let slice = "
        module Foo

        interface I {
            /// @param x: this is an x
            op1()\r

            op2(tag(1)
    x:\r
                    int32, tag(2) y: bool?,
            )
        }
\r
        enum E: int8 {}\r
        ";

        // Disable ANSI color codes.
        let options = SliceOptions {
            disable_color: true,
            ..Default::default()
        };

        // Parse the Slice file.
        let compilation_state = compile_from_strings(&[slice], Some(options), |_| {}, |_| {});

        let mut output: Vec<u8> = Vec::new();

        // Act
        compilation_state.emit_diagnostics(&mut output);

        // Assert
        let expected = "\
warning [IncorrectDocComment]: doc comment has a param tag for 'x', but there is no parameter by that name
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
9  |     x:\r
   | ------
10 |                     int32, tag(2) y: bool?,
   | -------------------------
   |
error [E010]: invalid enum 'E': enums must contain at least one enumerator
 --> string-0:14:9
   |
14 |         enum E: int8 {}\r
   |         ------
   |
";

        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn allow_all_warnings_flag() {
        let slice = "
            module Foo

            interface I {
                /// {@link Fake}
                /// @param x: this is an x
                op()
            }
        ";

        let options = SliceOptions {
            diagnostic_format: DiagnosticFormat::Json,
            allowed_warnings: vec!["All".to_owned()],
            ..Default::default()
        };

        // Parse the Slice file.
        let compilation_state = compile_from_strings(&[slice], Some(options), |_| {}, |_| {});

        let mut output: Vec<u8> = Vec::new();

        // Act
        compilation_state.emit_diagnostics(&mut output);

        // Assert
        assert_eq!("", String::from_utf8(output).unwrap());
    }

    #[test]
    fn allow_specific_warning_flag() {
        let slice = "
            module Foo

            interface I {
                /// {@link Fake}
                /// @param x: this is an x
                op()
            }
        ";

        // Set the output format to JSON.
        let options = SliceOptions {
            diagnostic_format: DiagnosticFormat::Json,
            allowed_warnings: vec!["BrokenDocLink".to_owned()],
            ..Default::default()
        };

        // Parse the Slice file.
        let compilation_state = compile_from_strings(&[slice], Some(options), |_| {}, |_| {});

        let mut output: Vec<u8> = Vec::new();

        // Act
        compilation_state.emit_diagnostics(&mut output);

        // Assert: Only one of the two warnings should be allowed.
        let expected = concat!(
            r#"{"message":"doc comment has a param tag for 'x', but there is no parameter by that name","severity":"warning","span":{"start":{"row":6,"col":21},"end":{"row":6,"col":43},"file":"string-0"},"notes":[],"error_code":"IncorrectDocComment"}"#,
            "\n",
        );
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn notes_with_same_span_as_diagnostic_suppressed() {
        // Arrange
        let slice = "
            encoding = Slice2
            module Foo
        ";

        // Disable ANSI color codes.
        let options = SliceOptions {
            disable_color: true,
            ..Default::default()
        };

        let mut compilation_state = compile_from_strings(&[slice], Some(options), |_| {}, |_| {});
        let mut output: Vec<u8> = Vec::new();

        // Report a diagnostic with a note that has the same span as the diagnostic.
        let span = Span {
            start: (2, 13).into(),
            end: (2, 39).into(),
            file: "string-0".to_owned(),
        };

        Diagnostic::new(Error::Syntax {
            message: "foo".to_owned(),
        })
        .set_span(&span)
        .add_note("bar", Some(&span))
        .report(&mut compilation_state.diagnostic_reporter);

        // Act
        compilation_state.emit_diagnostics(&mut output);

        // Assert
        let expected = "\
error [E002]: invalid syntax: foo
 --> string-0:2:13
  |
2 |             encoding = Slice2
  |             -----------------
  |
    = note: bar
";
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }
}
