// Copyright (c) ZeroC, Inc. All rights reserved.

use super::slice::*;
use super::traits::*;
use crate::ptr_util::OwnedPtr;

macro_rules! generate_definition_wrapper {
    ($($variant:ident),*) => {
        #[derive(Debug)]
        pub enum Definition {
            $($variant(OwnedPtr<$variant>),)*
        }

        impl Definition {
            pub fn borrow(&self) -> &dyn Entity {
                match self {
                    $(Self::$variant(x) => x.borrow(),)*
                }
            }

            pub unsafe fn borrow_mut(&mut self) -> &mut dyn Entity {
                match self {
                    $(Self::$variant(x) => x.borrow_mut(),)*
                }
            }
        }
    };
}

generate_definition_wrapper!(
    Module, Struct, Class, Exception, Interface, Enum, Trait, TypeAlias
);

macro_rules! generate_entities_wrapper {
    ($($variant:ident),*) => {
        #[derive(Debug)]
        pub enum Entities<'a> {
            $($variant(&'a $variant),)*
        }

        #[derive(Debug)]
        pub enum EntitiesMut<'a> {
            $($variant(&'a mut $variant),)*
        }

        $(
        impl AsEntities for $variant {
            fn concrete_entity(&self) -> Entities {
                Entities::$variant(self)
            }

            fn concrete_entity_mut(&mut self) -> EntitiesMut {
                EntitiesMut::$variant(self)
            }
        }
        )*
    };
}

pub trait AsEntities {
    fn concrete_entity(&self) -> Entities;
    fn concrete_entity_mut(&mut self) -> EntitiesMut;
}

generate_entities_wrapper!(
    Module, Struct, Class, Exception, DataMember, Interface, Operation, Parameter, Enum,
    Enumerator, Trait, TypeAlias
);

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

generate_types_wrapper!(
    Struct, Class, Exception, Interface, Enum, Trait, Sequence, Dictionary, Primitive
);
