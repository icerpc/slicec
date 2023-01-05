// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod attributes {

    mod slice_api {

        use crate::assert_errors;
        use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_diagnostics};
        use slice::diagnostics::{Error, ErrorKind, WarningKind};
        use slice::grammar::*;
        use test_case::test_case;

        #[test_case("Compact", ClassFormat::Compact ; "Compact")]
        #[test_case("Sliced", ClassFormat::Sliced; "Sliced")]
        fn format(format: &str, expected: ClassFormat) {
            // Arrange
            let slice = format!(
                "
                    module Test;

                    interface I
                    {{
                        [format({format})]
                        op(s: string) -> string;
                    }}
                "
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

                    interface I
                    {
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
            let args = arg.unwrap_or("");
            let slice = format!(
                "
                    module Test;

                    interface I
                    {{
                        [format{args}]
                        op(s: string) -> string;
                    }}
                "
            );

            // Act
            let diagnostic_reporter = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::CannotBeEmpty {
                member_identifier: "format attribute".to_owned(),
            });
            assert_errors!(diagnostic_reporter, [&expected]);
        }

        #[test]
        fn format_with_invalid_argument_fails() {
            // Arrange
            let slice = "
                module Test;

                interface I
                {
                    [format(Foo)]
                    op(s: string) -> string;
                }
            ";

            // Act
            let diagnostic_reporter = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::ArgumentNotSupported {
                argument_name: "Foo".to_owned(),
                method_name: "format attribute".to_owned(),
            })
            .add_note(
                "The valid arguments for the format attribute are `Compact` and `Sliced`",
                None,
            );

            assert_errors!(diagnostic_reporter, [&expected]);
        }

        #[test]
        fn deprecated() {
            // Arrange
            let slice = "
                module Test;

                interface I
                {
                    [deprecated]
                    op(s: string) -> string;
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
            assert!(operation.get_deprecation(false).is_some());
        }

        #[test]
        fn cannot_deprecate_parameters() {
            // Arrange
            let slice = "
                module Test;

                interface I
                {
                    op([deprecated] s: string) -> string;
                }
            ";

            // Act
            let diagnostic_reporter = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::DeprecatedAttributeCannotBeApplied {
                kind: "parameter(s)".to_owned(),
            });
            assert_errors!(diagnostic_reporter, [&expected]);
        }

        #[test]
        fn deprecated_can_contain_message() {
            // Arrange
            let slice = "
                module Test;

                interface I
                {
                    [deprecated(\"Deprecation message here\")]
                    op(s: string) -> string;
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();
            assert_eq!(
                operation.get_deprecation(false).unwrap().unwrap(),
                "Deprecation message here",
            );
        }

        #[test]
        fn deprecated_type_alias() {
            // Arrange
            let slice = "
                module Test;

                struct Foo
                {
                }

                [deprecated]
                typealias Bar = Foo;

                interface I
                {
                    op(s: Bar) -> string;
                }
            ";

            // Act
            let diagnostic_reporter = parse_for_diagnostics(slice);

            // Assert
            let expected = &crate::helpers::new_warning(WarningKind::UseOfDeprecatedEntity {
                identifier: "Bar".to_owned(),
                deprecation_reason: "".to_owned(),
            });
            assert_errors!(diagnostic_reporter, [&expected]);
        }

        #[test]
        fn deprecated_inheritance() {
            // Arrange
            let slice = "
            [deprecated]
            module Foo
            {
                struct Bar
                {
                }
            }

            module Test
            {
                struct Baz
                {
                    b: Foo::Bar,
                }
            }
            ";

            // Act
            let diagnostic_reporter = parse_for_diagnostics(slice);

            // Assert
            let expected = &crate::helpers::new_warning(WarningKind::UseOfDeprecatedEntity {
                identifier: "Bar".to_owned(),
                deprecation_reason: "".to_owned(),
            });
            assert_errors!(diagnostic_reporter, [&expected]);
        }

        #[test]
        fn cannot_use_deprecated_type() {
            // Arrange
            let slice = "
                    module Test;

                    [deprecated(\"Message here\")]
                    struct A
                    {
                    }

                    struct B
                    {
                        a: A,
                    }
                ";

            // Act
            let diagnostic_reporter = parse_for_diagnostics(slice);

            // Assert
            let expected = &crate::helpers::new_warning(WarningKind::UseOfDeprecatedEntity {
                identifier: "A".to_owned(),
                deprecation_reason: ": Message here".to_owned(),
            });
            assert_errors!(diagnostic_reporter, [&expected]);
        }

        #[test]
        fn cannot_inherit_from_deprecated_entity() {
            // Arrange
            let slice = "
                    module Test;

                    [deprecated]
                    interface A
                    {
                    }

                    interface B: A
                    {
                    }
                ";

            // Act
            let diagnostic_reporter = parse_for_diagnostics(slice);

            // Assert
            let expected = &crate::helpers::new_warning(WarningKind::UseOfDeprecatedEntity {
                identifier: "A".to_owned(),
                deprecation_reason: "".to_owned(),
            });
            assert_errors!(diagnostic_reporter, [&expected]);
        }

        #[test]
        fn compress() {
            // Arrange
            let slice = "
                module Test;

                interface I
                {
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

                interface I
                {
                    [compress(Foo)]
                    op(s: string) -> string;
                }
            ";

            // Act
            let diagnostic_reporter = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::ArgumentNotSupported {
                argument_name: "Foo".to_owned(),
                method_name: "compress attribute".to_owned(),
            })
            .add_note(
                "The valid arguments for the compress attribute are `Args` and `Return`",
                None,
            );
            assert_errors!(diagnostic_reporter, [&expected]);
        }

        #[test]
        fn cannot_compress_structs() {
            // Arrange
            let slice = "
                module Test;

                [compress()]
                struct S
                {
                    s: string,
                }
            ";

            // Act
            let diagnostic_reporter = parse_for_diagnostics(slice);

            // Assert
            let expected = Error::new(ErrorKind::CompressAttributeCannotBeApplied);
            assert_errors!(diagnostic_reporter, [&expected]);
        }

        #[test]
        fn compress_with_no_arguments() {
            // Arrange
            let slice = "
                module Test;

                interface I
                {
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

        #[test_case(
            "
            module Test;

            interface I
            {
                // The below doc comment will generate a warning
                /// A test operation. Similar to {@linked OtherOp}{}.
                [ignoreWarnings]
                op(s: string) -> string;
            }
            "; "simple"
        )]
        #[test_case(
            "
            [ignoreWarnings]
            module A
            {
                struct A1
                {
                    b: B::B1,
                }
            }
            module B
            {
                [deprecated]
                struct B1
                {
                }
            }
            "; "complex"
        )]
        #[test_case(
            "
            [ignoreWarnings]
            module A
            {
                struct A1
                {
                    b: sequence<B::B1>,
                }
            }
            module B
            {
                [deprecated]
                struct B1 {}
            }
            "; "complex with anonymous type"
        )]
        #[test_case(
            "
            [[ignoreWarnings]]
            module A
            {
                struct A1
                {
                    b: B::B1,
                }
            }
            module B
            {
                struct B1
                {
                }
            }
            "; "file level"
        )]
        fn ignore_warnings_attribute(slice: &str) {
            // Act
            let diagnostic_reporter = parse_for_diagnostics(slice);

            // Assert
            assert_errors!(diagnostic_reporter);
        }

        #[test]
        fn ignore_warnings_with_invalid_code() {
            // Arrange
            let slice = "
            module Test;

            interface I
            {
                [ignoreWarnings(W315, w001)]
                op(s: string) -> string;
            }
            ";

            // Act
            let diagnostic_reporter = parse_for_diagnostics(slice);

            // Assert
            let expected = [
                Error::new(ErrorKind::InvalidWarningCode {
                    code: "W315".to_owned(),
                }),
                Error::new(ErrorKind::InvalidWarningCode {
                    code: "w001".to_owned(),
                }),
            ];
            assert_errors!(diagnostic_reporter, expected);
        }

        #[test_case(
            "
            module Test;

            interface I
            {
                // The below doc comment will generate a warning
                /// A test operation. Similar to {@linked OtherOp}{}.
                /// @param b A test parameter.
                [ignoreWarnings(W006, W002)]
                op(s: string) -> string;
            }
            "; "entity"
        )]
        #[test_case(
            "
            [[ignoreWarnings(W006, W002)]]
            module Test;

            interface I
            {
                // The below doc comment will generate a warning
                /// A test operation. Similar to {@linked OtherOp}{}.
                /// @param b A test parameter.
                op(s: string) -> string;
            }
            "; "file level"
        )]
        fn ignore_warnings_attribute_args(slice: &str) {
            // Act
            let diagnostic_reporter = parse_for_diagnostics(slice);

            // Assert
            assert_errors!(diagnostic_reporter);
        }

        #[test_case(
            "
            module Test;

            interface I
            {
                /// @param x a parameter that should be used in ops
                /// @returns a result
                [ignoreWarnings(W003, W004)]
                op(s: string);
            }
            "; "entity"
        )]
        #[test_case(
            "
            [[ignoreWarnings(W003, W004)]]
            module Test;

            interface I
            {
                /// @param x a parameter that should be used in ops
                /// @returns a result
                [ignoreWarnings(W003, W004)]
                op(s: string);
            }
            "; "file level"
        )]
        // Test that if args are passed to ignoreWarnings, that only those warnings are ignored
        fn ignore_warnings_attribute_with_args_will_not_ignore_all_warnings(slice: &str) {
            // Act
            let diagnostic_reporter = parse_for_diagnostics(slice);

            // Assert
            let expected = &crate::helpers::new_warning(WarningKind::ExtraParameterInDocComment {
                identifier: "x".to_owned(),
            });

            debug_assert_eq!(expected.error_code(), "W002");
            assert_errors!(diagnostic_reporter, [&expected]);
        }
    }

    mod generalized_api {

        use crate::helpers::parsing_helpers::parse_for_ast;
        use slice::compile_from_strings;
        use slice::grammar::*;
        use test_case::test_case;

        #[test]
        fn foo_attribute() {
            // Arrange
            let slice = "
                module Test;

                interface I
                {
                    [foo::bar]
                    op(s: string) -> string;
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();

            let (directive, arguments) = operation
                .attributes(false)
                .iter()
                .find_map(|a| match &a.kind {
                    AttributeKind::Other { directive, arguments } if directive == "foo::bar" => {
                        Some((directive, arguments))
                    }
                    _ => None,
                })
                .unwrap();

            assert_eq!(directive, "foo::bar");
            assert_eq!(arguments.len(), 0);
        }

        #[test]
        fn foo_attribute_with_arguments() {
            // Arrange
            let slice = "
                module Test;

                interface I
                {
                    [foo::bar(a, b, c)]
                    op(s: string) -> string;
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();

            let (directive, arguments) = operation
                .attributes(false)
                .iter()
                .find_map(|a| match &a.kind {
                    AttributeKind::Other { directive, arguments } if directive == "foo::bar" => {
                        Some((directive, arguments))
                    }
                    _ => None,
                })
                .unwrap();

            assert_eq!(directive, "foo::bar");
            assert_eq!(arguments, &vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]);
        }

        #[test_case("a", &["a"]; "single argument")]
        #[test_case("\"a b c\"", &["a b c"]; "quoted argument")]
        #[test_case("a,b,c", &["a", "b", "c"]; "multiple arguments")]
        #[test_case("\"a, b, c\"", &["a, b, c"]; "quoted argument with comma")]
        fn attribute_parameters_multiple(input: &str, expected: &[&str]) {
            // Arrange
            let slice = format!(
                "
                    module Test;
                    interface I
                    {{
                        [foo::bar({input})]
                        op(s: string) -> string;
                    }}
                "
            );

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();

            match &operation.attributes[0].kind {
                AttributeKind::Other { arguments, .. } => {
                    for (i, v) in arguments.iter().enumerate() {
                        assert_eq!(v, expected.get(i).unwrap().to_owned());
                    }
                }
                _ => unreachable!(),
            }
        }

        #[test_case("a, \""; "quoted argument with unterminated string literal")]
        #[test_case("a, )"; "missing argument")]
        #[test_case("fizz buzz"; "unquoted argument with spaces")]
        fn attribute_with_invalid_parameters(input: &str) {
            // Arrange
            let slice = format!(
                "
                    module Test;
                    interface I
                    {{
                        [foo::bar({input})]
                        op(s: string) -> string;
                    }}
                "
            );

            // Act
            let errors = compile_from_strings(&[&slice], None).err();

            // Assert
            assert!(errors.is_some());
        }

        #[test]
        fn attribute_directives_can_be_slice_keywords() {
            // Arrange
            let slice = "
                [custom]
                module Test;
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let module = ast.find_element::<Module>("Test").unwrap();
            assert_eq!(module.attributes.len(), 1);
            assert!(matches!(
                &module.attributes[0].kind,
                AttributeKind::Other { directive, .. } if directive == "custom",
            ));
        }

        #[test]
        fn parent_attributes() {
            // Arrange
            let slice = r#"
                [attribute("A")]
                module A
                {
                    [attribute("B")]
                    module B
                    {
                        module C
                        {
                            [attribute("I")]
                            interface I
                            {
                                op(s: string) -> string;
                            }
                        }
                    }
                }
            "#;

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("A::B::C::I::op").unwrap();
            let parent_attributes = operation
                .attributes(true)
                .into_iter()
                .map(|a| match &a.kind {
                    AttributeKind::Other { directive, arguments } => (directive.as_str(), arguments),
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();

            assert_eq!(parent_attributes.len(), 3);
            assert_eq!(parent_attributes[0], ("attribute", &vec!["I".to_owned()]));
            assert_eq!(parent_attributes[1], ("attribute", &vec!["B".to_owned()]));
            assert_eq!(parent_attributes[2], ("attribute", &vec!["A".to_owned()]));
        }
    }
}
