// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod attributes {

    mod slice_api {

        use crate::helpers::parsing_helpers::parse_for_ast;
        use slice::grammar::*;
        use slice::parse_from_string;
        use test_case::test_case;

        #[test_case("Compact", ClassFormat::Compact)]
        #[test_case("Sliced", ClassFormat::Sliced)]
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
                format
            );

            // Act
            let ast = parse_for_ast(&slice);

            // Assert
            let operation_ptr = ast.find_typed_entity::<Operation>("Test::I::op").unwrap();
            let operation = operation_ptr.borrow();

            assert_eq!(operation.class_format(), expected);
        }

        #[test] // TODO: Using an invalid format should not panic but should be handled gracefully.
        fn format_with_invalid_format_fails() {
            // Arrange
            let slice = "
                module Test;

                interface I {
                    [format(Fake format)]
                    op(s: string) -> string;
                }
                ";

            // Act
            let errors = parse_from_string(slice).err();

            // Assert
            assert!(errors.is_some());
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
            let operation_ptr = ast.find_typed_entity::<Operation>("Test::I::op").unwrap();
            let operation = operation_ptr.borrow();

            assert!(operation.get_deprecated_attribute(false).is_some());
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
            let operation_ptr = ast.find_typed_entity::<Operation>("Test::I::op").unwrap();
            let operation = operation_ptr.borrow();

            assert!(operation.get_deprecated_attribute(false).is_some());
            assert_eq!(
                operation.get_deprecated_attribute(false).unwrap()[0],
                "Deprecation message here"
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
            let operation_ptr = ast.find_typed_entity::<Operation>("Test::I::op").unwrap();
            let operation = operation_ptr.borrow();

            assert!(operation.compress_arguments());
            assert!(operation.compress_return());
        }
    }

    mod generalized_api {

        use crate::helpers::parsing_helpers::parse_for_ast;
        use slice::grammar::*;

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
            let operation_ptr = ast.find_typed_entity::<Operation>("Test::I::op").unwrap();
            let operation = operation_ptr.borrow();

            assert!(operation.has_attribute("foo::bar", true));

            assert_eq!(operation.attributes[0].directive, "bar");
            assert_eq!(operation.attributes[0].prefixed_directive, "foo::bar");
            assert_eq!(operation.attributes[0].prefix, Some("foo".to_string()));
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
            let operation_ptr = ast.find_typed_entity::<Operation>("Test::I::op").unwrap();
            let operation = operation_ptr.borrow();

            assert!(operation.has_attribute("foo::bar", true));

            assert_eq!(operation.attributes[0].directive, "bar");
            assert_eq!(operation.attributes[0].prefixed_directive, "foo::bar");
            assert_eq!(operation.attributes[0].prefix, Some("foo".to_string()));
            assert_eq!(operation.attributes[0].arguments[0], "1");
            assert_eq!(operation.attributes[0].arguments[1], "2");
            assert_eq!(operation.attributes[0].arguments[2], "3");
        }
    }
}
