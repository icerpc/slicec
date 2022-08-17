// Copyright (c) ZeroC, Inc. All rights reserved.

use super::elements::*;
use super::traits::*;
use crate::utils::ptr_util::WeakPtr;

macro_rules! generate_definition_wrapper {
    ($($variant:ident),*) => {
        #[derive(Debug)]
        pub enum Definition {
            $($variant(WeakPtr<$variant>),)*
        }

        impl Definition {
            #[allow(clippy::should_implement_trait)]
            pub fn borrow(&self) -> &dyn Entity {
                match self {
                    $(Self::$variant(x) => x.borrow(),)*
                }
            }
        }
    };
}

generate_definition_wrapper!(Module, Struct, Class, Exception, Interface, Enum, Trait, CustomType, TypeAlias);

macro_rules! generate_types_wrapper {
    ($($variant:ident),*) => {
        #[derive(Debug)]
        pub enum Types<'a> {
            $($variant(&'a $variant),)*
        }

        #[derive(Debug)]
        pub enum TypesMut<'a> {
            $($variant(&'a mut $variant),)*
        }

        $(
        impl AsTypes for $variant {
            fn concrete_type(&self) -> Types {
                Types::$variant(self)
            }

            fn concrete_type_mut(&mut self) -> TypesMut {
                TypesMut::$variant(self)
            }
        }
        )*

        #[derive(Debug)]
        pub enum TypeRefs {
            $($variant(TypeRef<$variant>),)*
        }

        impl<T: Type + ?Sized> TypeRef<T> {
            pub fn concrete_typeref(&self) -> TypeRefs {
                match self.definition().concrete_type() {
                    $(Types::$variant(_) => TypeRefs::$variant(
                        self.downcast::<$variant>().ok().unwrap(),
                    ),)*
                }
            }
        }
    };
}

pub trait AsTypes {
    fn concrete_type(&self) -> Types;
    fn concrete_type_mut(&mut self) -> TypesMut;
}

generate_types_wrapper!(Struct, Class, Exception, Interface, Enum, Trait, CustomType, Sequence, Dictionary, Primitive);
