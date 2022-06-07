// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

mod scope_resolution {

    use crate::assert_errors;
    use crate::helpers::parsing_helpers::{parse_for_ast, parse_for_errors};
    use slice::grammar::*;

    #[test]
    fn file_level_modules_can_not_contain_sub_modules() {
        let slice = "
        module T;
        module S {}
        ";
        let error_reporter = parse_for_errors(slice);

        assert_errors!(error_reporter, [
            "file level modules cannot contain sub-modules",
            "file level module 'T' declared here",
        ]);
    }

    #[test]
    fn identifier_exists_in_module_and_submodule() {
        let slice = "
        module A
        {
            typealias S = int32;

            module B
            {
                struct S
                {
                    v: string,
                }
            }

            struct C
            {
                s1: S,
                s2: A::S,
                s3: B::S,
                s4: A::B::S,
            }
        }
        ";

        let ast = parse_for_ast(slice);

        let s1_ptr = ast.find_typed_entity::<DataMember>("A::C::s1").unwrap();
        let s2_ptr = ast.find_typed_entity::<DataMember>("A::C::s2").unwrap();
        let s3_ptr = ast.find_typed_entity::<DataMember>("A::C::s3").unwrap();
        let s4_ptr = ast.find_typed_entity::<DataMember>("A::C::s4").unwrap();

        let s1_type = s1_ptr.borrow().data_type();
        let s2_type = s2_ptr.borrow().data_type();
        let s3_type = s3_ptr.borrow().data_type();
        let s4_type = s4_ptr.borrow().data_type();

        assert!(matches!(s1_type.concrete_type(), Types::Primitive(Primitive::Int32)));
        assert!(matches!(s2_type.concrete_type(), Types::Primitive(Primitive::Int32)));
        assert!(matches!(s3_type.concrete_type(), Types::Struct(_)));
        assert!(matches!(s4_type.concrete_type(), Types::Struct(_)));
    }

    #[test]
    fn identifier_exists_in_module_and_parent_module() {
        let slice = "
        module A
        {
            typealias S = int32;

            module B
            {
                typealias S = string;

                struct C
                {
                    s1: S,
                    s2: B::S,
                    s3: A::B::S,
                    s4: A::S,
                }
            }
        }
        ";

        let ast = parse_for_ast(slice);

        let s1_ptr = ast.find_typed_entity::<DataMember>("A::B::C::s1").unwrap();
        let s2_ptr = ast.find_typed_entity::<DataMember>("A::B::C::s2").unwrap();
        let s3_ptr = ast.find_typed_entity::<DataMember>("A::B::C::s3").unwrap();
        let s4_ptr = ast.find_typed_entity::<DataMember>("A::B::C::s4").unwrap();

        let s1_type = s1_ptr.borrow().data_type();
        let s2_type = s2_ptr.borrow().data_type();
        let s3_type = s3_ptr.borrow().data_type();
        let s4_type = s4_ptr.borrow().data_type();

        assert!(matches!(s1_type.concrete_type(), Types::Primitive(Primitive::String)));
        assert!(matches!(s2_type.concrete_type(), Types::Primitive(Primitive::String)));
        assert!(matches!(s3_type.concrete_type(), Types::Primitive(Primitive::String)));
        assert!(matches!(s4_type.concrete_type(), Types::Primitive(Primitive::Int32)));
    }

    #[test]
    fn identifier_exists_in_multiple_parent_modules() {
        let slice = "
        module A
        {
            typealias S = int32;

            module B
            {

                struct S
                {
                    v: string,
                }

                module B
                {

                    struct C
                    {
                        s1: S,
                        s2: B::S,
                        s3: A::S,
                    }
                }
            }
        }
        ";

        let ast = parse_for_ast(slice);

        let s1_ptr = ast.find_typed_entity::<DataMember>("A::B::B::C::s1").unwrap();
        let s2_ptr = ast.find_typed_entity::<DataMember>("A::B::B::C::s2").unwrap();
        let s3_ptr = ast.find_typed_entity::<DataMember>("A::B::B::C::s3").unwrap();

        let s1_type = s1_ptr.borrow().data_type();
        let s2_type = s2_ptr.borrow().data_type();
        let s3_type = s3_ptr.borrow().data_type();

        assert!(matches!(s1_type.concrete_type(), Types::Struct(_)));
        assert!(matches!(s2_type.concrete_type(), Types::Struct(_)));
        assert!(matches!(s3_type.concrete_type(), Types::Primitive(Primitive::Int32)));
    }

    #[test]
    fn identifier_exists_in_multiple_modules_with_common_partial_scope() {
        let slice = "
        module A
        {
            module B
            {
                typealias S = string;

                module A
                {
                    module B
                    {
                        typealias S = int32;

                        struct C
                        {
                            s1: A::B::S,
                            s2: ::A::B::S,
                        }
                    }
                }
            }

            struct C
            {
                s1: A::B::S,
                s2: ::A::B::S,
            }
        }
        ";

        let ast = parse_for_ast(slice);

        let nested_s1_ptr = ast.find_typed_entity::<DataMember>("A::B::A::B::C::s1").unwrap();
        let nested_s2_ptr = ast.find_typed_entity::<DataMember>("A::B::A::B::C::s2").unwrap();
        let s1_ptr = ast.find_typed_entity::<DataMember>("A::C::s1").unwrap();
        let s2_ptr = ast.find_typed_entity::<DataMember>("A::C::s2").unwrap();

        let nested_s1_type = nested_s1_ptr.borrow().data_type();
        let nested_s2_type = nested_s2_ptr.borrow().data_type();
        let s1_type = s1_ptr.borrow().data_type();
        let s2_type = s2_ptr.borrow().data_type();

        assert!(matches!(
            nested_s1_type.concrete_type(),
            Types::Primitive(Primitive::Int32),
        ));
        assert!(matches!(
            nested_s2_type.concrete_type(),
            Types::Primitive(Primitive::String),
        ));

        assert!(matches!(s1_type.concrete_type(), Types::Primitive(Primitive::String)));
        assert!(matches!(s2_type.concrete_type(), Types::Primitive(Primitive::String)));
    }

    #[test]
    fn interface_has_same_identifier_as_module() {
        // Arrange
        let slice = "
        module A
        {
            module B
            {
            }

            interface B
            {
            }

            struct S
            {
                b: B,
            }
        }
        ";

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, [
            "redefinition of B",
            "B was previously defined here",
        ]);
    }

    #[test]
    fn relative_scope_is_module_before_interface() {
        // Arrange
        let slice = "
        module A
        {
            module B
            {
                module C
                {
                    struct S
                    {
                        c: C,
                    }
                }
            }

            interface C {}
        }
        ";

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, [
            "module `A::B::C` cannot be used as a type",
        ]);
    }

    #[test]
    fn missing_type_should_fail() {
        // Arrange
        let slice = "
        module A
        {
            struct C
            {
                b: Nested::C,
            }
        }
        ";

        // Act
        let error_reporter = parse_for_errors(slice);

        // Assert
        assert_errors!(error_reporter, [
            "No entity with the identifier 'Nested::C' could be found in this scope.",
        ]);
    }
}
