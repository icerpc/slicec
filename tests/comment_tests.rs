// Copyright (c) ZeroC, Inc.

mod test_helpers;

mod comments {

    use crate::test_helpers::*;
    use slicec::diagnostics::{Diagnostic, Error, Lint};
    use slicec::grammar::*;
    use test_case::test_case;

    #[test]
    fn single_line_doc_comment() {
        // Arrange
        let slice = "
            module tests

            /// This is a single line doc comment.
            interface MyInterface {}
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let interface_def = ast.find_element::<Interface>("tests::MyInterface").unwrap();

        let interface_doc = interface_def.comment().unwrap();
        assert_eq!(interface_doc.span.start, (4, 13).into());
        assert_eq!(interface_doc.span.end, (4, 51).into());

        let overview = &interface_doc.overview.as_ref().unwrap();
        assert_eq!(overview.span.start, (4, 16).into());
        assert_eq!(overview.span.end, (4, 51).into());

        let message = &overview.message;
        assert_eq!(message.len(), 2);
        let MessageComponent::Text(text) = &message[0] else { panic!() };
        assert_eq!(text, "This is a single line doc comment.");
        let MessageComponent::Text(newline) = &message[1] else { panic!() };
        assert_eq!(newline, "\n");
    }

    #[test]
    fn multi_line_doc_comment() {
        // Arrange
        let slice = "
            module tests

            /// This is a
            /// multiline doc comment.
            interface MyInterface {}
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let interface_def = ast.find_element::<Interface>("tests::MyInterface").unwrap();

        let interface_doc = interface_def.comment().unwrap();
        assert_eq!(interface_doc.span.start, (4, 13).into());
        assert_eq!(interface_doc.span.end, (5, 39).into());

        let overview = &interface_doc.overview.as_ref().unwrap();
        assert_eq!(overview.span.start, (4, 16).into());
        assert_eq!(overview.span.end, (5, 39).into());

        let message = &overview.message;
        assert_eq!(message.len(), 4);
        let MessageComponent::Text(text) = &message[0] else { panic!() };
        assert_eq!(text, "This is a");
        let MessageComponent::Text(newline) = &message[1] else { panic!() };
        assert_eq!(newline, "\n");
        let MessageComponent::Text(text) = &message[2] else { panic!() };
        assert_eq!(text, "multiline doc comment.");
        let MessageComponent::Text(newline) = &message[3] else { panic!() };
        assert_eq!(newline, "\n");
    }

    #[test]
    fn doc_comments_params() {
        // Arrange
        let slice = "
            module tests

            interface TestInterface {
                /// @param testParam: My test param
                testOp(testParam: string)
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let operation = ast.find_element::<Operation>("tests::TestInterface::testOp").unwrap();

        let param_tags = &operation.comment().unwrap().params;
        assert_eq!(param_tags.len(), 1);

        let param_tag = &param_tags[0];
        assert_eq!(param_tag.span.start, (5, 21).into());
        assert_eq!(param_tag.span.end, (5, 52).into());

        let identifier = &param_tag.identifier;
        assert_eq!(identifier.value, "testParam");
        assert_eq!(identifier.span.start, (5, 28).into());
        assert_eq!(identifier.span.end, (5, 37).into());

        let message = &param_tag.message;
        assert_eq!(message.len(), 2);
        let MessageComponent::Text(text) = &message[0] else { panic!() };
        assert_eq!(text, "My test param");
    }

    #[test]
    fn doc_comments_returns() {
        // Arrange
        let slice = "
            module tests

            interface TestInterface {
                /// @returns bool
                testOp(testParam: string) -> bool
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let operation = ast.find_element::<Operation>("tests::TestInterface::testOp").unwrap();

        let returns_tags = &operation.comment().unwrap().returns;
        assert_eq!(returns_tags.len(), 1);

        let returns_tag = &returns_tags[0];
        assert_eq!(returns_tag.span.start, (5, 21).into());
        assert_eq!(returns_tag.span.end, (5, 34).into());

        let identifier = &returns_tag.identifier.as_ref().unwrap();
        assert_eq!(identifier.value, "bool");
        assert_eq!(identifier.span.start, (5, 30).into());
        assert_eq!(identifier.span.end, (5, 34).into());

        let message = &returns_tag.message;
        assert!(message.is_empty());
    }

    #[test]
    fn doc_comments_not_supported_on_modules() {
        // Arrange
        let slice = "
            /// This is a module comment.
            module tests
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Error::Syntax {
            message: "doc comments cannot be applied to modules".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn doc_comment_not_supported_on_params_and_returns() {
        // Arrange
        let slice = "
                module tests

                interface I {
                    testOp(
                        /// comment on param
                        testParam: string,
                    )
                    testOpTwo() -> (
                        /// comment on return
                        foo: string,
                        bar: string,
                    )
                }
            ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = [
            Diagnostic::new(Error::Syntax {
                message: "doc comments cannot be applied to parameters".to_owned(),
            }),
            Diagnostic::new(Error::Syntax {
                message: "doc comments cannot be applied to return members".to_owned(),
            }),
        ];
        check_diagnostics(diagnostics, expected);
    }

    #[test]
    fn operation_with_correct_doc_comments() {
        // Arrange
        let slice = "
            module tests

            exception MyException {}

            interface TestInterface {
                /// @param testParam1: A string param
                /// @returns: bool
                /// @throws MyException: Some message about why testOp throws
                testOp(testParam1: string) -> bool throws MyException
            }
        ";

        // Act/Assert
        assert_parses(slice);
    }

    #[test]
    fn doc_comment_throws() {
        // Arrange
        let slice = "
            module tests

            interface TestInterface {
                /// @throws: Message about my thrown thing.
                testOp(testParam: string) -> bool
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let operation = ast.find_element::<Operation>("tests::TestInterface::testOp").unwrap();

        let throws_tags = &operation.comment().unwrap().throws;
        assert_eq!(throws_tags.len(), 1);

        let throws_tag = &throws_tags[0];
        assert_eq!(throws_tag.span.start, (5, 21).into());
        assert_eq!(throws_tag.span.end, (5, 60).into());

        assert!(throws_tag.thrown_type().is_none());

        let message = &throws_tag.message;
        assert_eq!(message.len(), 2);
        let MessageComponent::Text(text) = &message[0] else { panic!() };
        assert_eq!(text, "Message about my thrown thing.");
    }

    #[test]
    fn doc_comments_throws_specific_type() {
        // Arrange
        let slice = "
            module tests

            exception MyThrownThing {}

            interface TestInterface {
                /// @throws MyThrownThing: Message about my thrown thing.
                testOp(testParam: string) -> bool
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let operation = ast.find_element::<Operation>("tests::TestInterface::testOp").unwrap();

        let throws_tags = &operation.comment().unwrap().throws;
        assert_eq!(throws_tags.len(), 1);

        let throws_tag = &throws_tags[0];
        assert_eq!(throws_tag.span.start, (7, 21).into());
        assert_eq!(throws_tag.span.end, (7, 74).into());

        let thrown_type = throws_tag.thrown_type().unwrap().unwrap();
        assert_eq!(thrown_type.module_scoped_identifier(), "tests::MyThrownThing");

        let message = &throws_tag.message;
        assert_eq!(message.len(), 2);
        let MessageComponent::Text(text) = &message[0] else { panic!() };
        assert_eq!(text, "Message about my thrown thing.");
    }

    #[test]
    fn doc_comments_see() {
        // Arrange
        let slice = "
            module tests

            interface TestInterface {
                /// @see MySee
                testOp(testParam: string) -> bool
            }
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let operation = ast.find_element::<Operation>("tests::TestInterface::testOp").unwrap();

        let see_tags = &operation.comment().unwrap().see;
        assert_eq!(see_tags.len(), 1);

        let see_tag = &see_tags[0];
        assert_eq!(see_tag.span.start, (5, 21).into());
        assert_eq!(see_tag.span.end, (5, 31).into());

        let Err(link_identifier) = see_tag.linked_entity() else { panic!() };
        assert_eq!(link_identifier.value, "MySee");
        assert_eq!(link_identifier.span.start, (5, 26).into());
        assert_eq!(link_identifier.span.end, (5, 31).into());
    }

    #[test_case("/* This is a block comment. */"; "block comment")]
    #[test_case("/*\n* This is a multiline block comment.\n */"; "multi-line block comment")]
    #[test_case("// This is a comment."; "comment")]
    fn non_doc_comments_are_ignored(comment: &str) {
        // Arrange
        let slice = format!(
            "
                module tests

                {comment}
                interface MyInterface {{}}
            "
        );

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let interface_def = ast.find_element::<Interface>("tests::MyInterface").unwrap();
        let interface_doc = interface_def.comment();

        assert!(interface_doc.is_none());
    }

    #[test]
    fn doc_comments_must_start_with_exactly_3_slashes() {
        // Arrange
        let slice = "
            module Test

            //// This is not a doc comment.
            struct Foo {}
        ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let struct_def = ast.find_element::<Struct>("Test::Foo").unwrap();
        let struct_doc = struct_def.comment();

        assert!(struct_doc.is_none());
    }

    #[test]
    fn doc_comment_linked_identifiers() {
        // Arrange
        let slice = "
            module tests

            /// This comment is for {@link TestStruct}
            struct TestStruct {}
            ";

        // Act
        let ast = parse_for_ast(slice);

        // Assert
        let struct_def = ast.find_element::<Struct>("tests::TestStruct").unwrap();
        let overview = &struct_def.comment().unwrap().overview;
        let message = &overview.as_ref().unwrap().message;

        assert_eq!(message.len(), 3);
        let MessageComponent::Text(text) = &message[0] else { panic!() };
        assert_eq!(text, "This comment is for ");
        let MessageComponent::Link(link) = &message[1] else { panic!() };
        assert_eq!(link.linked_entity().unwrap().identifier(), "TestStruct");
        let MessageComponent::Text(newline) = &message[2] else { panic!() };
        assert_eq!(newline, "\n");
    }

    #[test]
    fn unknown_doc_comment_tag() {
        // Arrange
        let slice = "
            module tests

            /// A test struct. Similar to {@linked OtherStruct}{}.
            struct TestStruct {}
            ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Lint::MalformedDocComment {
            message: "unknown doc comment tag 'linked'".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn missing_doc_comment_linked_identifiers() {
        // Arrange
        let slice = "
            module tests

            /// A test struct. Similar to {@link OtherStruct}.
            struct TestStruct {}
            ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Lint::BrokenDocLink {
            message: "no element named 'OtherStruct' exists in scope".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test_case("bool", "primitive types"; "primitive")]
    #[test_case("tests", "modules"; "module")]
    fn doc_comment_links_to_invalid_element(link_identifier: &str, kind: &str) {
        // Arrange
        let slice = format!(
            "
            module tests

            /// A test struct, should probably use {{@link {link_identifier}}}.
            struct TestStruct {{}}
            "
        );

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Lint::BrokenDocLink {
            message: format!("{kind} cannot be linked to"),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn param_tag_is_rejected_for_operations_with_no_parameters() {
        // Arrange
        let slice = "
            module tests

            interface I {
                /// @param foo: this parameter doesn't exist.
                op()
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Lint::IncorrectDocComment {
            message: "comment has a 'param' tag for 'foo', but its operation has no parameter with that name"
                .to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn param_tag_is_rejected_if_its_identifier_does_not_match_a_parameters() {
        // Arrange
        let slice = "
            module tests

            interface I {
                /// @param foo: this parameter doesn't exist.
                op(bar: bool)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Lint::IncorrectDocComment {
            message: "comment has a 'param' tag for 'foo', but its operation has no parameter with that name"
                .to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test_case("returns"; "unnamed tag")]
    #[test_case("returns foo"; "named tag")]
    fn returns_tag_is_rejected_for_operations_that_return_nothing(returns_tag: &str) {
        // Arrange
        let slice = format!(
            "
            module tests

            interface I {{
                /// @{returns_tag}: this tag is invalid.
                op()
            }}
            ",
        );

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Lint::IncorrectDocComment {
            message: "comment has a 'returns' tag, but its operation does not return anything".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn named_returns_tag_is_rejected_for_operations_that_return_an_unnamed_type() {
        // Arrange
        let slice = "
            module tests

            interface I {
                /// @returns foo: this tag is invalid.
                op() -> bool
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Lint::IncorrectDocComment {
            message: "comment has a 'returns' tag for 'foo', but its operation doesn't return anything with that name"
                .to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn named_returns_tag_is_rejected_if_its_identifier_does_not_match_a_return_tuple_elements() {
        // Arrange
        let slice = "
            module tests

            interface I {
                /// @returns foo: this tag is invalid.
                op() -> (alice: bool, bob: bool)
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Lint::IncorrectDocComment {
            message: "comment has a 'returns' tag for 'foo', but its operation doesn't return anything with that name"
                .to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test_case("throws"; "unnamed tag")]
    #[test_case("throws Foo"; "named tag")]
    fn throws_tag_is_rejected_for_operations_that_do_not_throw(throws_tag: &str) {
        // Arrange
        let slice = format!(
            "
            module tests

            exception Foo {{}}

            interface I {{
                /// @{throws_tag}: this tag is invalid.
                op()
            }}
            ",
        );

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Lint::IncorrectDocComment {
            message: "comment has a 'throws' tag, but its operation does not throw anything".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn throws_tag_is_rejected_if_it_names_an_unthrowable_type() {
        // Arrange
        let slice = "
            module tests

            struct Foo {}

            interface I {
                /// @throws Foo: this type isn't an exception`.
                op()
            }
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = [
            Diagnostic::new(Lint::IncorrectDocComment {
                message: "comment has a 'throws' tag for 'Foo', but it is not a throwable type".to_owned(),
            }),
            Diagnostic::new(Lint::IncorrectDocComment {
                message: "comment has a 'throws' tag, but its operation does not throw anything".to_owned(),
            }),
        ];
        check_diagnostics(diagnostics, expected);
    }

    #[test]
    fn named_throws_tag_is_rejected_if_its_types_does_not_match_the_thrown_type() {
        // Arrange
        let slice = format!(
            "
            module tests

            exception E {{}}
            exception Foo {{}}

            interface I {{
                /// @throws Foo: this isn't the type that is thrown.
                op() throws E
            }}
            "
        );

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Lint::IncorrectDocComment {
            message: "comment has a 'throws' tag for 'Foo', but it's operation doesn't throw this exception".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn param_tags_can_only_be_used_with_operations() {
        // Arrange
        let slice = "
            module tests

            /// @param foo: bad tag.
            struct Foo {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Lint::IncorrectDocComment {
            message: "comment has a 'param' tag, but only operations can have parameters".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn returns_tags_can_only_be_used_with_operations() {
        // Arrange
        let slice = "
            module tests

            /// @returns: bad tag.
            struct Foo {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Lint::IncorrectDocComment {
            message: "comment has a 'returns' tag, but only operations can return".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }

    #[test]
    fn throws_tags_can_only_be_used_with_operations() {
        // Arrange
        let slice = "
            module tests

            /// @throws: bad tag.
            struct Foo {}
        ";

        // Act
        let diagnostics = parse_for_diagnostics(slice);

        // Assert
        let expected = Diagnostic::new(Lint::IncorrectDocComment {
            message: "comment has a 'throws' tag, but only operations can throw".to_owned(),
        });
        check_diagnostics(diagnostics, [expected]);
    }
}
