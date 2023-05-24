// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod attributes {
    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error, Warning};

    mod allow {
        use super::*;
        use test_case::test_case;

        #[test]
        fn local_allow_attribute_parses() {
            // Arrange
            let slice = "
                module Test

                [allow(All)]
                struct S {}
            ";

            // Act/Assert
            assert_parses(slice);
        }

        #[test]
        fn file_level_allow_attribute_parses() {
            // Arrange
            let slice = "[[allow(All)]]";

            // Act/Assert
            assert_parses(slice);
        }

        #[test]
        fn allow_with_invalid_argument() {
            // Arrange
            let slice = "[[allow(Fake)]]";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::ArgumentNotSupported {
                argument: "Fake".to_owned(),
                directive: "allow".to_owned(),
            });
            check_diagnostics(diagnostics, [expected]);
        }

        #[test_case("All"; "all")]
        #[test_case("IncorrectDocComment"; "specific")]
        fn allow_with_valid_arguments(argument: &str) {
            // Arrange
            let slice = format!("[[allow({argument})]]");

            // Act/Assert
            assert_parses(slice);
        }

        #[test]
        fn ensure_allow_can_take_multiple_arguments() {
            // Arrange
            let slice = "[[allow(BrokenDocLink, Deprecated)]]";

            // Act/Assert
            assert_parses(slice);
        }

        #[test]
        fn ensure_allow_requires_arguments() {
            // Arrange
            let slice = "[[allow]]";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::MissingRequiredArgument {
                argument: "allow(<arguments>)".to_owned(),
            });
            check_diagnostics(diagnostics, [expected]);
        }

        #[test_case("All", []; "all")]
        #[test_case("Deprecated", [1, 2]; "deprecated")]
        #[test_case("BrokenDocLink", [0, 2]; "broken_link")]
        #[test_case("IncorrectDocComment", [0, 1]; "incorrect_doc_comment")]
        fn allow_only_specified_warnings<const L: usize>(arguments: &str, expected_indexes: [usize; L]) {
            // Arrange
            let slice = format!(
                "
                [[allow({arguments})]]
                module Test

                /// {{@link fake}}
                /// @throws
                [deprecated(\"test\")]
                struct S {{}}

                struct UseS {{
                    s: S
                }}
                "
            );

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let mut all_warnings = vec![
                Diagnostic::new(Warning::Deprecated {
                    identifier: "S".to_owned(),
                    reason: Some("test".to_owned()),
                }),
                Diagnostic::new(Warning::BrokenDocLink {
                    message: "no element named 'fake' exists in scope".to_owned(),
                }),
                Diagnostic::new(Warning::IncorrectDocComment {
                    message: "doc comment indicates that struct 'S' throws, however, only operations can throw"
                        .to_owned(),
                }),
            ];
            // Filter out any warning that should be ignored by the supplied test arguments.
            let mut index = 0;
            all_warnings.retain(|_| {
                index += 1;
                expected_indexes.contains(&(index - 1))
            });
            let expected: [Diagnostic; L] = all_warnings.try_into().unwrap();

            // Check that only the correct warnings were emitted.
            check_diagnostics(diagnostics, expected);
        }
    }

    mod slice_api {

        use crate::test_helpers::*;
        use slicec::diagnostics::{Diagnostic, Error, Warning};
        use slicec::grammar::*;
        use slicec::slice_file::Span;
        use test_case::test_case;

        #[test]
        fn sliced_format() {
            // Arrange
            let slice = "
                module Test

                interface I {
                    [slicedFormat(Args, Return)]
                    op(s: string) -> string
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();

            assert!(operation.slice_classes_in_arguments());
            assert!(operation.slice_classes_in_return());
        }

        #[test]
        fn sliced_format_with_invalid_arguments_fails() {
            // Arrange
            let slice = "
                module Test

                interface I {
                    [slicedFormat(Foo)]
                    op(s: string) -> string
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::ArgumentNotSupported {
                argument: "Foo".to_owned(),
                directive: "slicedFormat".to_owned(),
            })
            .add_note("'Args' and 'Return' are the only valid arguments", None);

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn sliced_format_only_works_on_operations() {
            // Arrange
            let slice = "
                module Test

                [slicedFormat]
                struct S {
                    s: string
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::UnexpectedAttribute {
                attribute: "slicedFormat".to_owned(),
            })
            .set_span(&Span::new((4, 18).into(), (4, 30).into(), "string-0"))
            .add_note("the slicedFormat attribute can only be applied to operations", None);

            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn sliced_format_with_no_arguments() {
            // Arrange
            let slice = "
                module Test

                interface I {
                    [slicedFormat]
                    op(s: string) -> string
                }
            ";

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let operation = ast.find_element::<Operation>("Test::I::op").unwrap();

            assert!(!operation.slice_classes_in_arguments());
            assert!(!operation.slice_classes_in_return());
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
            assert!(operation.get_deprecation().is_some());
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
            .add_note("parameters can not be individually deprecated", None);

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
                operation.get_deprecation().unwrap().unwrap(),
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
            let expected = Diagnostic::new(Warning::Deprecated {
                identifier: "Bar".to_owned(),
                reason: None,
            });
            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn deprecated_is_not_allowed_on_modules() {
            // Arrange
            let slice = "
                [deprecated]
                module Foo

                struct Bar {}
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::UnexpectedAttribute {
                attribute: "deprecated".to_owned(),
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
            let expected = Diagnostic::new(Warning::Deprecated {
                identifier: "A".to_owned(),
                reason: Some("Message here".to_owned()),
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
            let expected = Diagnostic::new(Warning::Deprecated {
                identifier: "A".to_owned(),
                reason: None,
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
            .add_note("'Args' and 'Return' are the only valid arguments", None);

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

        #[test]
        fn non_repeatable_attributes_error() {
            // Arrange
            let slice = "
                module Test

                interface Foo {
                    [compress(Args)]
                    [compress(Return)]
                    op()
                }
            ";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::AttributeIsNotRepeatable {
                attribute: "compress".to_owned(),
            });
            check_diagnostics(diagnostics, [expected]);
        }

        #[test]
        fn type_ref_attributes_error() {
            // Arrange
            let slice = "
                module Test

                struct Foo {
                    a: [oneway] string
                }";

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::UnexpectedAttribute {
                attribute: "oneway".to_owned(),
            });

            check_diagnostics(diagnostics, [expected]);
        }

        #[test_case("[deprecated] string"; "non nested")]
        #[test_case("sequence<[deprecated] string>"; "nested")]
        fn attributes_on_anonymous_types_are_rejected(alias_type: &str) {
            let slice = format!(
                "
                module Test

                typealias AnAlias = {alias_type}
            "
            );

            let diagnostics = parse_for_diagnostics(slice);

            let expected = Diagnostic::new(Error::UnexpectedAttribute {
                attribute: "deprecated".to_owned(),
            });

            check_diagnostics(diagnostics, [expected]);
        }

        #[test_case("oneway", "struct Foo {}"; "oneway on struct")]
        #[test_case("slicedFormat", "exception Foo {}"; "slicedFormat on exception")]
        fn non_common_attributes_rejected(attribute: &str, slice_type: &str) {
            let slice = format!(
                "
                module Test

                [{attribute}]
                {slice_type}
            "
            );

            let diagnostics = parse_for_diagnostics(slice);

            let expected = Diagnostic::new(Error::UnexpectedAttribute {
                attribute: attribute.to_owned(),
            });

            check_diagnostics(diagnostics, [expected]);
        }
    }

    mod generalized_api {

        use crate::test_helpers::*;
        use slicec::diagnostics::{Diagnostic, Error};
        use slicec::grammar::*;
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
                .attributes()
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
                .attributes()
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
            let compilation_state = parse(slice, None);

            // Assert
            assert!(compilation_state.diagnostic_reporter.has_errors());
        }

        #[test]
        fn parent_attributes() {
            // Arrange
            let slice = r#"
                [test::attribute("A")]
                module A

                [test::attribute("I")]
                interface I {
                    op([test::attribute("S")] s: string) -> string
                }
            "#;

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let parameter = ast.find_element::<Parameter>("A::I::op::s").unwrap();
            let parent_attributes = parameter
                .all_attributes()
                .concat()
                .into_iter()
                .map(|a| match &a.kind {
                    AttributeKind::Other { directive, arguments } => (directive.as_str(), arguments),
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>();

            assert_eq!(parent_attributes.len(), 3);
            assert_eq!(parent_attributes[0], ("test::attribute", &vec!["S".to_owned()]));
            assert_eq!(parent_attributes[1], ("test::attribute", &vec!["I".to_owned()]));
            assert_eq!(parent_attributes[2], ("test::attribute", &vec!["A".to_owned()]));
        }

        #[test_case("foo"; "plain_attribute")]
        #[test_case("custom"; "slice_keyword")]
        fn unknown_attributes_are_rejected(directive: &str) {
            // Arrange
            let slice = format!(
                "
                    [{directive}]
                    module Test
                "
            );

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::UnexpectedAttribute {
                attribute: directive.to_owned(),
            });

            check_diagnostics(diagnostics, [expected]);
        }

        #[test_case("::", "::"; "colon_colon")]
        #[test_case("::foo", "::"; "leading_colon_colon")]
        #[test_case("foo::", "]"; "trailing_colon_colon")]
        fn attribute_with_bogus_directive_is_rejected(directive: &str, found: &str) {
            // Arrange
            let slice = format!(
                "
                    [{directive}]
                    module Test
                "
            );

            // Act
            let diagnostics = parse_for_diagnostics(slice);

            // Assert
            let expected = Diagnostic::new(Error::Syntax {
                message: format!("expected one of 'identifier', but found '{found}'"),
            });

            check_diagnostics(diagnostics, [expected]);
        }

        #[test_case("cs::custom"; "cs")]
        #[test_case("foo::custom"; "foo")]
        fn unknown_language_attributes_are_not_rejected(attribute: &str) {
            // Arrange
            let slice = format!(
                "
                    [{attribute}]
                    module Test
                "
            );

            // Act
            let ast = parse_for_ast(slice);

            // Assert
            let module = ast.find_element::<Module>("Test").unwrap();
            assert_eq!(module.attributes.len(), 1);
            assert!(matches!(
                &module.attributes[0].borrow().kind,
                AttributeKind::Other { directive, .. } if directive == attribute,
            ));
        }
    }
}
