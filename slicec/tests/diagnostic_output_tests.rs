// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod output {
    use crate::test_helpers::parse;
    use slicec::diagnostic_emitter::DiagnosticEmitter;
    use slicec::slice_options::{DiagnosticFormat, SliceOptions};

    #[test]
    fn output_to_json() {
        let slice = r#"
        module Foo

        interface I {
            /// @param x: this is an x
            op()
        }

        enum E : int8 {}
        "#;

        // Set the output format to JSON.
        let options = SliceOptions {
            diagnostic_format: DiagnosticFormat::Json,
            ..Default::default()
        };

        // Parse the Slice file.
        let state = parse(slice, Some(&options));
        let diagnostics = state.diagnostics.into_updated(&state.ast, &state.files, &options);

        let mut output: Vec<u8> = Vec::new();
        let mut emitter = DiagnosticEmitter::new(&mut output, &options, &state.files);

        // Act
        emitter.emit_diagnostics(diagnostics).unwrap();

        // Assert
        let expected = concat!(
            r#"{"message":"comment has a 'param' tag for 'x', but operation 'op' has no parameter with that name","severity":"warning","span":{"start":{"row":5,"col":17},"end":{"row":5,"col":25},"file":"string-0"},"notes":[],"error_code":"IncorrectDocComment"}"#,
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
            op1()

            op2(tag(1)
    x:
                    int32, tag(2) y: bool?,
            )
        }

        enum E : int8 {}
        ";

        // Disable ANSI color codes.
        let options = SliceOptions {
            disable_color: true,
            ..Default::default()
        };

        // Parse the Slice file.
        let state = parse(slice, Some(&options));
        let diagnostics = state.diagnostics.into_updated(&state.ast, &state.files, &options);

        let mut output: Vec<u8> = Vec::new();
        let mut emitter = DiagnosticEmitter::new(&mut output, &options, &state.files);

        // Act
        emitter.emit_diagnostics(diagnostics).unwrap();

        // Assert
        let expected = "\
warning [IncorrectDocComment]: comment has a 'param' tag for 'x', but operation 'op1' has no parameter with that name
 --> string-0:5:17
  |
5 |             /// @param x: this is an x
  |                 --------
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
14 |         enum E : int8 {}
   |         ------
   |
";

        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn allow_all_lints_flag() {
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
            allowed_lints: vec!["All".to_owned()],
            ..Default::default()
        };

        // Parse the Slice file.
        let state = parse(slice, Some(&options));
        let diagnostics = state.diagnostics.into_updated(&state.ast, &state.files, &options);

        let mut output: Vec<u8> = Vec::new();
        let mut emitter = DiagnosticEmitter::new(&mut output, &options, &state.files);

        // Act
        emitter.emit_diagnostics(diagnostics).unwrap();

        // Assert
        assert_eq!("", String::from_utf8(output).unwrap());
    }

    #[test]
    fn allow_specific_lint_flag() {
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
            allowed_lints: vec!["BrokenDocLink".to_owned()],
            ..Default::default()
        };

        // Parse the Slice file.
        let state = parse(slice, Some(&options));
        let diagnostics = state.diagnostics.into_updated(&state.ast, &state.files, &options);

        let mut output: Vec<u8> = Vec::new();
        let mut emitter = DiagnosticEmitter::new(&mut output, &options, &state.files);

        // Act
        emitter.emit_diagnostics(diagnostics).unwrap();

        // Assert: Only one of the two lints should be allowed.
        let expected = concat!(
            r#"{"message":"comment has a 'param' tag for 'x', but operation 'op' has no parameter with that name","severity":"warning","span":{"start":{"row":6,"col":21},"end":{"row":6,"col":29},"file":"string-0"},"notes":[],"error_code":"IncorrectDocComment"}"#,
            "\n",
        );
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn crlf_line_endings() {
        let slice = "module Foo \r\n   enum\r\n E\r : uint8\r\n{}\r\n\r";

        // Disable ANSI color codes.
        let options = SliceOptions {
            disable_color: true,
            ..Default::default()
        };

        let state = parse(slice, Some(&options));
        let diagnostics = state.diagnostics.into_updated(&state.ast, &state.files, &options);

        let mut output: Vec<u8> = Vec::new();
        let mut emitter = DiagnosticEmitter::new(&mut output, &options, &state.files);

        // Act
        emitter.emit_diagnostics(diagnostics).unwrap();

        // Assert
        let expected = "\
error [E010]: invalid enum 'E': enums must contain at least one enumerator
 --> string-0:2:4
  |
2 |    enum
  |    ----
3 |  E\r : uint8
  | --
  |
";
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }
}
