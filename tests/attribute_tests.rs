// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod attributes {

    mod slice_api {

        use crate::assert_errors_new;
        use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_errors};
        use slice::errors::{ErrorKind, LogicKind};
        use slice::grammar::*;
        use test_case::test_case;

        #[test_case("Compact", ClassFormat::Compact ; "Compact")]
        #[test_case("Sliced", ClassFormat::Sliced; "Sliced")]
        fn format(format: &str, expected: ClassFormat) {
            // Arrange
            let slice = format!(
                "
                    module Test;

                    interface I {{
                        [format({})]
                        op(s: string) -> string;
                    }}
                ",
                format,
            );

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
            assert_eq!(operation.class_format(), expected);
        }

        #[test]
        fn not_specifying_format_uses_compact_as_default() {
            // Arrange
            let slice = "
                    module Test;

                    interface I {
                        op(s: string) -> string;
                    }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
            assert_eq!(operation.class_format(), ClassFormat::Compact);
        }

        #[test_case(Some("()") ; "empty parenthesis")]
        #[test_case(None; "No parenthesis or arguments")]
        fn format_with_no_argument_fails(arg: Option<&str>) {
            // Arrange
            let slice = format!(
                "
                    module Test;

                    interface I {{
                        [format{}]
                        op(s: string) -> string;
                    }}
                ",
                arg.unwrap_or(""),
            );

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            let expected: ErrorKind = LogicKind::CannotBeEmpty("format attribute").into();
            assert_errors_new!(error_reporter, [&expected]);
        }

        #[test]
        fn format_with_invalid_argument_fails() {
            // Arrange
            let slice = "
                module Test;

                interface I {
                    [format(Foo)]
                    op(s: string) -> string;
                }
            ";
            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            let expected = [
                LogicKind::ArgumentNotSupported("Foo".to_owned(), "format attribute".to_owned()).into(),
                ErrorKind::new_note("The valid arguments for the format attribute are `Compact` and `Sliced`"),
            ];
            assert_errors_new!(error_reporter, expected);
        }

        #[test]
        fn deprecated() {
            // Arrange
            let slice = "
                module Test;

                interface I {
                    [deprecated]
                    op(s: string) -> string;
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
            assert!(operation.get_deprecated_attribute(false).is_some());
        }

        #[test]
        fn cannot_deprecate_parameters() {
            // Arrange
            let slice = "
                module Test;

                interface I {
                    op([deprecated] s: string) -> string;
                }
            ";

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            let expected: ErrorKind = LogicKind::DeprecatedAttributeCannotBeApplied("parameter(s)".to_owned()).into();
            assert_errors_new!(error_reporter, [&expected]);
        }

        #[test]
        fn cannot_deprecate_data_members() {
            // Arrange
            let slice = "
                module Test;

                struct S {
                    [deprecated]
                    s: string,
                }
            ";

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            let expected: ErrorKind = LogicKind::DeprecatedAttributeCannotBeApplied("data member(s)".to_owned()).into();
            assert_errors_new!(error_reporter, [&expected]);
        }

        #[test]
        fn deprecated_can_contain_message() {
            // Arrange
            let slice = "
                module Test;

                interface I {
                    [deprecated(\"Deprecation message here\")]
                    op(s: string) -> string;
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();

            assert!(operation.get_deprecated_attribute(false).is_some());
            assert_eq!(
                operation.get_deprecated_attribute(false).unwrap()[0],
                "Deprecation message here",
            );
        }

        #[test]
        fn compress() {
            // Arrange
            let slice = "
                module Test;

                interface I {
                    [compress(Args, Return)]
                    op(s: string) -> string;
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();

            assert!(operation.compress_arguments());
            assert!(operation.compress_return());
        }

        #[test]
        fn compress_with_invalid_arguments_fails() {
            // Arrange
            let slice = "
                module Test;

                interface I {
                    [compress(Foo)]
                    op(s: string) -> string;
                }
            ";

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            let expected = [
                LogicKind::ArgumentNotSupported("Foo".to_owned(), "compress attribute".to_owned()).into(),
                ErrorKind::new_note("The valid argument(s) for the compress attribute are `Args` and `Return`"),
            ];
            assert_errors_new!(error_reporter, expected);
        }

        #[test]
        fn cannot_compress_structs() {
            // Arrange
            let slice = "
                module Test;

                [compress()]
                struct S {
                    s: string,
                }
            ";

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            let expected: ErrorKind = LogicKind::CompressAttributeCannotBeApplied.into();
            assert_errors_new!(error_reporter, [&expected]);
        }

        #[test]
        fn compress_with_no_arguments() {
            // Arrange
            let slice = "
                module Test;

                interface I {
                    [compress()]
                    op(s: string) -> string;
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();

            assert!(!operation.compress_arguments());
            assert!(!operation.compress_return());
        }
    }

    mod generalized_api {

        use crate::assert_errors;
        use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_errors};
        use slice::grammar::*;
        use slice::parse_from_string;
        use test_case::test_case;

        #[test]
        fn foo_attribute() {
            // Arrange
            let slice = "
                module Test;

                interface I {
                    [foo::bar]
                    op(s: string) -> string;
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();

            assert!(operation.has_attribute("foo::bar", true));
            assert_eq!(operation.attributes[0].directive, "bar");
            assert_eq!(operation.attributes[0].prefixed_directive, "foo::bar");
            assert_eq!(operation.attributes[0].prefix, Some("foo".to_owned()));
            assert_eq!(operation.attributes[0].arguments.len(), 0);
        }

        #[test]
        fn foo_attribute_with_arguments() {
            // Arrange
            let slice = "
                module Test;

                interface I {
                    [foo::bar(1, 2, 3)]
                    op(s: string) -> string;
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();

            assert!(operation.has_attribute("foo::bar", true));
            assert_eq!(operation.attributes[0].directive, "bar");
            assert_eq!(operation.attributes[0].prefixed_directive, "foo::bar");
            assert_eq!(operation.attributes[0].prefix, Some("foo".to_owned()));
            assert_eq!(operation.attributes[0].arguments[0], "1");
            assert_eq!(operation.attributes[0].arguments[1], "2");
            assert_eq!(operation.attributes[0].arguments[2], "3");
        }

        #[test_case("a", &["a"]; "single argument")]
        #[test_case("a,b,c", &["a", "b", "c"]; "multiple arguments")]
        #[test_case("\"a b c\"", &["a b c"]; "quoted argument")]
        #[test_case("\"a, b, c\"", &["a, b, c"]; "quoted argument with comma")]
        fn attribute_parameters(input: &str, expected: &[&str]) {
            // Arrange
            let slice = format!(
                "
                    module Test;
                    interface I {{
                        [foo::bar({input})]
                        op(s: string) -> string;
                    }}
                ",
                input = input,
            );

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();

            for (i, v) in operation.attributes[0].arguments.iter().enumerate() {
                assert_eq!(v, expected.get(i).unwrap().to_owned());
            }
        }

        #[test_case("a, \""; "quoted argument with comma and trailing comma")]
        #[test_case("a, )"; "quoted argument with comma and trailing parenthesis")]
        fn attribute_with_invalid_parameters(input: &str) {
            // Arrange
            let slice = format!(
                "
                    module Test;
                    interface I {{
                        [foo::bar({input})]
                        op(s: string) -> string;
                    }}
                ",
                input = input,
            );

            // Act
            let errors = parse_from_string(&slice).err();

            // Assert
            assert!(errors.is_some());
        }

        #[test]
        #[ignore] // TODO: Currently panics with "expected operation" error. Should be fixed
                  // in parser.
        fn foo_attribute_with_spaces_fails() {
            // Arrange
            let slice = "
                module Test;

                interface I {
                    [foo::bar(abcdefgh)]
                    op(s: string) -> string;
                }
            ";

            // Act
            let error_reporter = parse_for_errors(slice);

            // Assert
            assert_errors!(error_reporter, [
                "", // Should be error here
            ]);
        }
    }
}
