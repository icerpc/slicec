// Copyright (c) ZeroC, Inc. All rights reserved.

pub mod helpers;

use crate::helpers::parsing_helpers::parse_for_ast;
use slice::grammar::*;

mod scope_resolution {

    use super::*;

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

        assert_matches!(s1_type.concrete_type(), Types::Primitive, Primitive::Int32);
        assert_matches!(s2_type.concrete_type(), Types::Primitive, Primitive::Int32);
        assert_matches!(s3_type.concrete_type(), Types::Struct);
        assert_matches!(s4_type.concrete_type(), Types::Struct);
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

        assert_matches!(s1_type.concrete_type(), Types::Primitive, Primitive::String);
        assert_matches!(s2_type.concrete_type(), Types::Primitive, Primitive::String);
        assert_matches!(s3_type.concrete_type(), Types::Primitive, Primitive::String);
        assert_matches!(s4_type.concrete_type(), Types::Primitive, Primitive::Int32);
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

        let s1_ptr = ast
            .find_typed_entity::<DataMember>("A::B::B::C::s1")
            .unwrap();
        let s2_ptr = ast
            .find_typed_entity::<DataMember>("A::B::B::C::s2")
            .unwrap();
        let s3_ptr = ast
            .find_typed_entity::<DataMember>("A::B::B::C::s3")
            .unwrap();

        let s1_type = s1_ptr.borrow().data_type();
        let s2_type = s2_ptr.borrow().data_type();
        let s3_type = s3_ptr.borrow().data_type();

        assert_matches!(s1_type.concrete_type(), Types::Struct);
        assert_matches!(s2_type.concrete_type(), Types::Struct);
        assert_matches!(s3_type.concrete_type(), Types::Primitive, Primitive::Int32);
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

        let nested_s1_ptr = ast
            .find_typed_entity::<DataMember>("A::B::A::B::C::s1")
            .unwrap();
        let nested_s2_ptr = ast
            .find_typed_entity::<DataMember>("A::B::A::B::C::s2")
            .unwrap();
        let s1_ptr = ast.find_typed_entity::<DataMember>("A::C::s1").unwrap();
        let s2_ptr = ast.find_typed_entity::<DataMember>("A::C::s2").unwrap();

        let nested_s1_type = nested_s1_ptr.borrow().data_type();
        let nested_s2_type = nested_s2_ptr.borrow().data_type();
        let s1_type = s1_ptr.borrow().data_type();
        let s2_type = s2_ptr.borrow().data_type();

        assert_matches!(
            nested_s1_type.concrete_type(),
            Types::Primitive,
            Primitive::Int32
        );
        assert_matches!(
            nested_s2_type.concrete_type(),
            Types::Primitive,
            Primitive::String
        );

        assert_matches!(s1_type.concrete_type(), Types::Primitive, Primitive::String);
        assert_matches!(s2_type.concrete_type(), Types::Primitive, Primitive::String);
    }

    #[test]
    fn interface_has_same_identifier_as_module() {
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
                b: B
            }
        }
        ";

        let ast = parse_for_ast(slice);
        let b_ptr = ast.find_typed_entity::<DataMember>("A::S::b").unwrap();
        let b_type = b_ptr.borrow().data_type();

        assert_matches!(b_type.concrete_type(), Types::Interface);
    }

    #[test]
    #[ignore] // TODO: This test is broken
    fn relative_scope_is_module_before_interface() {
        let slice = "
        module A
        {
            module B
            {
                module C
                {
                    struct S
                    {
                        c: C
                    }
                }
            }

            interface C {}
        }
        ";

        let ast = parse_for_ast(slice);

        let b_ptr = ast
            .find_typed_entity::<DataMember>("A::B::C::S::c")
            .unwrap();
        let b_type = b_ptr.borrow().data_type();

        assert_matches!(b_type.concrete_type(), Types::Interface);
    }

    #[test]
    #[ignore] // TODO: This test is broken
    fn missing_type_should_fail() {
        let slice = "
        module A
        {
            struct C
            {
                b: Nested::C
            }
        }
        ";

        let ast = parse_for_ast(slice);

        let b_ptr = ast.find_typed_entity::<Interface>("A::B::I").unwrap();
        let b_def = b_ptr.borrow();

        assert_eq!(b_def.all_base_interfaces().len(), 1);
    }
}
