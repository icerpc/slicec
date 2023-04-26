// Copyright (c) ZeroC, Inc.

mod attributes {

    mod slice_api {

        use slice::diagnostics::{Diagnostic, Error, Warning};
        use slice::grammar::*;
        use slice::slice_file::Span;
        use slice::test_helpers::*;
        use test_case::test_case;

        #[test_case("Compact", ClassFormat::Compact ; "Compact")]
        #[test_case("Sliced", ClassFormat::Sliced; "Sliced")]
        fn format(format: &str, expected: ClassFormat) {
            // Arrange
            let slice = format!(
                "
                    module Test

                    interface I {{
                        [format({format})]
                        op(s: string) -> string
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
                    module Test

                    interface I {
                        op(s: string) -> string
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
                    module Test

                    interface I {{
                        [format{args}]
                        op(s: string) -> string
                    }}
                "
            );

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::MissingRequiredArgument {
                argument: r#"format(<arguments>)"#.to_owned(),
            });
            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn format_with_invalid_argument_fails() {
            // Arrange
            let slice = "
                module Test

                interface I {
                    [format(Foo)]
                    op(s: string) -> string
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::ArgumentNotSupported {
                argument: "Foo".to_owned(),
                directive: "format".to_owned(),
            })
            .add_note(
                "The valid arguments for the format attribute are 'Compact' and 'Sliced'",
                None,
            );

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn deprecated() {
            // Arrange
            let slice = "
                module Test

                interface I {
                    [deprecated]
                    op(s: string) -> string
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
                module Test

                interface I {
                    op([deprecated] s: string) -> string
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::UnexpectedAttribute {
                attribute: "deprecated".to_owned(),
            })
            .set_span(&Span::new((5, 25).into(), (5, 35).into(), "string-0"))
            .add_note("individual parameters cannot be deprecated", None);

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn deprecated_can_contain_message() {
            // Arrange
            let slice = "
                module Test

                interface I {
                    [deprecated(\"Deprecation message here\")]
                    op(s: string) -> string
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
                module Test

                struct Foo {}

                [deprecated]
                typealias Bar = Foo

                interface I {
                    op(s: Bar) -> string
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Warning::UseOfDeprecatedEntity {
                identifier: "Bar".to_owned(),
                deprecation_reason: "".to_owned(),
            });
            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn deprecated_inheritance() {
            // Arrange
            let slice = "
            [deprecated]
            module Foo {
                struct Bar {}
            }

            module Test {
                struct Baz {
                    b: Foo::Bar
                }
            }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Warning::UseOfDeprecatedEntity {
                identifier: "Bar".to_owned(),
                deprecation_reason: "".to_owned(),
            });
            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn cannot_use_deprecated_type() {
            // Arrange
            let slice = "
                    module Test

                    [deprecated(\"Message here\")]
                    struct A {}

                    struct B {
                        a: A
                    }
                ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Warning::UseOfDeprecatedEntity {
                identifier: "A".to_owned(),
                deprecation_reason: ": Message here".to_owned(),
            });
            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn cannot_inherit_from_deprecated_entity() {
            // Arrange
            let slice = "
                    module Test

                    [deprecated]
                    interface A {}

                    interface B: A {}
                ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Warning::UseOfDeprecatedEntity {
                identifier: "A".to_owned(),
                deprecation_reason: "".to_owned(),
            });
            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn compress() {
            // Arrange
            let slice = "
                module Test

                interface I {
                    [compress(Args, Return)]
                    op(s: string) -> string
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
                module Test

                interface I {
                    [compress(Foo)]
                    op(s: string) -> string
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::ArgumentNotSupported {
                argument: "Foo".to_owned(),
                directive: "compress".to_owned(),
            })
            .add_note(
                "The valid arguments for the compress attribute are 'Args' and 'Return'",
                None,
            );

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn cannot_compress_structs() {
            // Arrange
            let slice = "
                module Test

                [compress()]
                struct S {
                    s: string
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::UnexpectedAttribute {
                attribute: "compress".to_owned(),
            })
            .set_span(&Span::new((4, 18).into(), (4, 28).into(), "string-0"))
            .add_note(
                "the compress attribute can only be applied to interfaces and operations",
                None,
            );

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn compress_with_no_arguments() {
            // Arrange
            let slice = "
                module Test

                interface I {
                    [compress()]
                    op(s: string) -> string
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
            module Test

            interface I {
                // The below doc comment will generate a warning
                /// A test operation. Similar to {@linked OtherOp}{}.
                [allow]
                op(s: string) -> string
            }
            "; "simple"
        )]
        #[test_case(
            "
            [allow]
            module A {
                struct A1 {
                    b: B::B1
                }
            }
            module B {
                [deprecated]
                struct B1 {}
            }
            "; "complex"
        )]
        #[test_case(
            "
            [allow]
            module A {
                struct A1 {
                    b: sequence<B::B1>
                }
            }
            module B {
                [deprecated]
                struct B1 {}
            }
            "; "complex with anonymous type"
        )]
        #[test_case(
            "
            [[allow]]
            module A {
                struct A1 {
                    b: B::B1
                }
            }
            module B {
                struct B1 {}
            }
            "; "file level"
        )]
        fn allow_attribute(slice: &str) {
            assert_parses(slice);
        }

        #[test]
        fn allow_with_invalid_code() {
            // Arrange
            let slice = "
            module Test

            interface I {
                [allow(W315, w001)]
                op(s: string) -> string
            }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = [
                Diagnostic::new(Error::InvalidWarningCode {
                    code: "W315".to_owned(),
                }),
                Diagnostic::new(Error::InvalidWarningCode {
                    code: "w001".to_owned(),
                }),
            ];
            check_diagnostics(diagnostics, expected);
        }

        #[test_case(
            "
            module Test

            interface I {
                // The below doc comment will generate a warning
                /// A test operation. Similar to {@linked OtherOp}{}.
                /// @param b: A test parameter.
                [allow(W002, W003)]
                op(s: string) -> string
            }
            "; "entity"
        )]
        #[test_case(
            "
            [[allow(W002, W003)]]
            module Test

            interface I {
                // The below doc comment will generate a warning
                /// A test operation. Similar to {@linked OtherOp}{}.
                /// @param b: A test parameter.
                op(s: string) -> string
            }
            "; "file level"
        )]
        fn allow_attribute_args(slice: &str) {
            assert_parses(slice);
        }

        #[test_case(
            "
            module Test

            interface I {
                /// @param x: a parameter that should be used in ops
                /// @returns: a result
                [allow(W004, W005)]
                op(s: string)
            }
            "; "entity"
        )]
        #[test_case(
            "
            [[allow(W004, W005)]]
            module Test

            interface I {
                /// @param x: a parameter that should be used in ops
                /// @returns: a result
                [allow(W004, W005)]
                op(s: string)
            }
            "; "file level"
        )]
        // Test that if args are passed to allow, that only those warnings are allowed
        fn allow_attribute_with_args_will_not_allow_all_warnings(slice: &str) {
            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Warning::ExtraParameterInDocComment {
                identifier: "x".to_owned(),
            });
            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn non_repeatable_attributes_error() {
            // Act
            let slice = "
                module Test

                interface Foo {
                    [compress(Args)]
                    [compress(Return)]
                    op()
                }
            ";

            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::AttributeIsNotRepeatable {
                attribute: "compress".to_owned(),
            });
            check_diagnostics(diagnostics, [expected]);
        }
    }

    mod generalized_api {

        use slice::compile_from_strings;
        use slice::grammar::*;
        use slice::test_helpers::parse_for_ast;
        use test_case::test_case;

        #[test]
        fn foo_attribute() {
            // Arrange
            let slice = "
                module Test

                interface I {
                    [foo::bar]
                    op(s: string) -> string
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
                module Test

                interface I {
                    [foo::bar(a, b, c)]
                    op(s: string) -> string
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
                    module Test

                    interface I {{
                        [foo::bar({input})]
                        op(s: string) -> string
                    }}
                "
            );

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();

            match &operation.attributes[0].borrow().kind {
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
                    module Test

                    interface I {{
                        [foo::bar({input})]
                        op(s: string) -> string
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
                module Test
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let module = ast.find_element::<Module>("Test").unwrap();
            assert_eq!(module.attributes.len(), 1);
            assert!(matches!(
                &module.attributes[0].borrow().kind,
                AttributeKind::Other { directive, .. } if directive == "custom",
            ));
        }

        #[test]
        fn parent_attributes() {
            // Arrange
            let slice = r#"
                [attribute("A")]
                module A {
                    [attribute("B")]
                    module B {
                        module C {
                            [attribute("I")]
                            interface I {
                                op(s: string) -> string
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
