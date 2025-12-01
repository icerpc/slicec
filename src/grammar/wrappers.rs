// Copyright (c) ZeroC, Inc.

use super::elements::*;
use super::traits::*;
use crate::slice_file::SliceFile;
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

generate_definition_wrapper!(Struct, Class, Exception, Interface, Enum, CustomType, TypeAlias);

macro_rules! generate_entities_wrapper {
    ($($variant:ident),*) => {
        #[derive(Debug)]
        pub enum Entities<'a> {
            $($variant(&'a $variant),)*
        }

        $(
        impl AsEntities for $variant {
            fn concrete_entity(&self) -> Entities<'_> {
                Entities::$variant(self)
            }
        }
        )*
    };
}

pub trait AsEntities {
    fn concrete_entity(&self) -> Entities<'_>;
}

generate_entities_wrapper!(
    Struct, Class, Exception, Field, Interface, Operation, Parameter, Enum, Enumerator, CustomType, TypeAlias
);

macro_rules! generate_attributables_wrapper {
    ($($variant:ident),*) => {
        #[derive(Debug)]
        pub enum Attributables<'a> {
            $($variant(&'a $variant),)*
        }

        $(
        impl AsAttributables for $variant {
            fn concrete_attributable(&self) -> Attributables<'_> {
                Attributables::$variant(self)
            }
        }
        )*
    };
}

pub trait AsAttributables {
    fn concrete_attributable(&self) -> Attributables<'_>;
}

generate_attributables_wrapper!(
    Module, Struct, Class, Exception, Field, Interface, Operation, Parameter, Enum, Enumerator, CustomType, TypeAlias,
    TypeRef, SliceFile
);

macro_rules! generate_types_wrapper {
    ($($variant:ident),*) => {
        #[derive(Debug)]
        pub enum Types<'a> {
            $($variant(&'a $variant),)*
        }

        $(
        impl AsTypes for $variant {
            fn concrete_type(&self) -> Types<'_> {
                Types::$variant(self)
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
    fn concrete_type(&self) -> Types<'_>;
}

generate_types_wrapper!(Struct, Class, Enum, CustomType, ResultType, Sequence, Dictionary, Primitive);
