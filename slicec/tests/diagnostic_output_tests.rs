// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod output {
    use crate::test_helpers::parse;
    use slicec::compilation_state::CompilationState;
    use slicec::diagnostic_emitter::DiagnosticEmitter;
    use slicec::diagnostics::{Diagnostic, Error, Lint};
    use slicec::slice_options::{DiagnosticFormat, SliceOptions};

    #[test]
    fn output_to_json() {
        // Arrange
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
        let diagnostics = state.get_annotated_diagnostics(&options);

        let mut output: Vec<u8> = Vec::new();
        let mut emitter = DiagnosticEmitter::new(&mut output, &options);

        // Act
        emitter.emit_diagnostics(&diagnostics).unwrap();

        // Assert
        let expected = concat!(
            r#"{"message":"incorrect doc comment: comment has a 'param' tag for 'x', but operation 'op' has no parameter with that name","severity":"warning","snippet":{"span":{"start":{"row":5,"col":17},"end":{"row":5,"col":25},"file":"string-0"},"text":"  |\n5 |             /// @param x: this is an x\n  |                 --------\n  |"},"notes":[],"error_code":"IncorrectDocComment"}"#,
            "\n",
            r#"{"message":"invalid enum 'E': enums must contain at least one enumerator","severity":"error","snippet":{"span":{"start":{"row":9,"col":9},"end":{"row":9,"col":15},"file":"string-0"},"text":"  |\n9 |         enum E : int8 {}\n  |         ------\n  |"},"notes":[],"error_code":"E008"}"#,
            "\n",
        );
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn output_to_console() {
        // Arrange
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
        let diagnostics = state.get_annotated_diagnostics(&options);

        let mut output: Vec<u8> = Vec::new();
        let mut emitter = DiagnosticEmitter::new(&mut output, &options);

        // Act
        emitter.emit_diagnostics(&diagnostics).unwrap();

        // Assert
        let expected = "\
warning [IncorrectDocComment]: incorrect doc comment: comment has a 'param' tag for 'x', but operation 'op1' has no parameter with that name
 --> string-0:5:17
  |
5 |             /// @param x: this is an x
  |                 --------
  |
error [E016]: invalid tag on member 'x': tagged members must be optional
 --> string-0:8:17
   |
8  |             op2(tag(1)
   |                 ------
9  |     x:
   | ------
10 |                     int32, tag(2) y: bool?,
   | -------------------------
   |
error [E008]: invalid enum 'E': enums must contain at least one enumerator
 --> string-0:14:9
   |
14 |         enum E : int8 {}
   |         ------
   |
";

        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn duplicate_diagnostics_are_filtered_out() {
        // Arrange

        // All of these diagnostics are equal.
        let diagnostic1_1 = Diagnostic::from_lint(Lint::Other {
            message: "This is a test".to_owned(),
        });
        let diagnostic1_2 = Diagnostic::from_lint(Lint::Other {
            message: "This is a test".to_owned(),
        });
        let diagnostic1_3 = Diagnostic::from_lint(Lint::Other {
            message: "This is a test".to_owned(),
        });
        // This diagnostic is unique, there should be no other diagnostics equal to it.
        let diagnostic2_1 = Diagnostic::from_lint(Lint::Other {
            message: "This is also a test".to_owned(),
        });
        // These 2 diagnostics are equal.
        let diagnostic3_1 = Diagnostic::from_error(Error::CompactStructCannotBeEmpty);
        let diagnostic3_2 = Diagnostic::from_error(Error::CompactStructCannotBeEmpty);
        // This diagnostic should not be equal to diagnostic 3, since this one has a note.
        let diagnostic4_1 = Diagnostic::from_error(Error::CompactStructCannotBeEmpty)
            .add_note("This diagnostic is different because it has a note", None);

        // Manually set the diagnostics in our compilation state.
        let mut state = CompilationState::create();
        state.diagnostics.extend([
            diagnostic1_1,
            diagnostic1_2,
            diagnostic2_1,
            diagnostic3_1,
            diagnostic4_1,
            diagnostic1_3,
            diagnostic3_2,
        ]);

        // Act
        let converted_diagnostics = state.get_annotated_diagnostics(&SliceOptions::default());

        // Assert
        assert_eq!(converted_diagnostics.len(), 4);
        assert_eq!(converted_diagnostics[0].message, "This is a test");
        assert_eq!(converted_diagnostics[1].message, "This is also a test");
        assert_eq!(converted_diagnostics[2].message, "compact structs must be non-empty");
        assert_eq!(converted_diagnostics[3].message, "compact structs must be non-empty");

        let (warnings, errors) = DiagnosticEmitter::get_totals(&converted_diagnostics);
        assert_eq!(warnings, 2);
        assert_eq!(errors, 2);
    }

    #[test]
    fn allow_all_lints_flag() {
        // Arrange
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
        let diagnostics = state.get_annotated_diagnostics(&options);

        let mut output: Vec<u8> = Vec::new();
        let mut emitter = DiagnosticEmitter::new(&mut output, &options);

        // Act
        emitter.emit_diagnostics(&diagnostics).unwrap();

        // Assert
        assert_eq!("", String::from_utf8(output).unwrap());
    }

    #[test]
    fn allow_specific_lint_flag() {
        // Arrange
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
        let diagnostics = state.get_annotated_diagnostics(&options);

        let mut output: Vec<u8> = Vec::new();
        let mut emitter = DiagnosticEmitter::new(&mut output, &options);

        // Act
        emitter.emit_diagnostics(&diagnostics).unwrap();

        // Assert: Only one of the two lints should be allowed.
        let expected = concat!(
            r#"{"message":"incorrect doc comment: comment has a 'param' tag for 'x', but operation 'op' has no parameter with that name","severity":"warning","snippet":{"span":{"start":{"row":6,"col":21},"end":{"row":6,"col":29},"file":"string-0"},"text":"  |\n6 |                 /// @param x: this is an x\n  |                     --------\n  |"},"notes":[],"error_code":"IncorrectDocComment"}"#,
            "\n",
        );
        assert_eq!(expected, String::from_utf8(output).unwrap());
    }

    #[test]
    fn crlf_line_endings() {
        // Arrange
        let slice = "module Foo \r\n   enum\r\n E\r : uint8\r\n{}\r\n\r";

        // Disable ANSI color codes.
        let options = SliceOptions {
            disable_color: true,
            ..Default::default()
        };

        let state = parse(slice, Some(&options));
        let diagnostics = state.get_annotated_diagnostics(&options);

        let mut output: Vec<u8> = Vec::new();
        let mut emitter = DiagnosticEmitter::new(&mut output, &options);

        // Act
        emitter.emit_diagnostics(&diagnostics).unwrap();

        // Assert
        let expected = "\
error [E008]: invalid enum 'E': enums must contain at least one enumerator
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
