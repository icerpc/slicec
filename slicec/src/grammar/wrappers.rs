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

macro_rules! generate_elements_wrapper {
    ($($variant:ident),*) => {
        #[derive(Debug)]
        pub enum Elements<'a> {
            $($variant(&'a $variant),)*
        }

        #[derive(Debug)]
        pub enum ElementsMut<'a> {
            $($variant(&'a mut $variant),)*
        }
    };
}

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
    };
}

macro_rules! generate_typerefs_wrapper {
    ($($variant:ident),*) => {
        #[derive(Debug)]
        pub enum TypeRefs {
            $($variant(TypeRef<$variant>),)*
        }

        impl<T: Element + ?Sized> TypeRef<T> {
            pub fn concrete_type_ref(&self) -> TypeRefs {
                match self.concrete_element() {
                    $(Elements::$variant(_) => {
                        // Clone the TypeRef, but downcast it's pointer to the concrete type.
                        // TODO cloning this is expensive. There may be a better way to implement.
                        let downcasted = TypeRef {
                            type_string: self.type_string.clone(),
                            definition: self.definition.clone().downcast::<$variant>().unwrap(),
                            is_optional: self.is_optional,
                            scope: self.scope.clone(),
                            attributes: self.attributes.clone(),
                            location: self.location.clone(),
                        };
                        TypeRefs::$variant(downcasted)
                    })*
                    _ => panic!("Impossible TypeRef value: {:?}", self.concrete_element()),
                }
            }
        }
    };
}

macro_rules! implement_as_elements {
    ($type:ident) => {
        impl AsElements for $type {
            fn concrete_element(&self) -> Elements {
                Elements::$type(self)
            }

            fn concrete_element_mut(&mut self) -> ElementsMut {
                ElementsMut::$type(self)
            }
        }
    };
}

macro_rules! implement_as_types {
    ($type:ident) => {
        implement_as_elements!($type);

        impl AsTypes for $type {
            fn concrete_type(&self) -> Types {
                Types::$type(self)
            }

            fn concrete_type_mut(&mut self) -> TypesMut {
                TypesMut::$type(self)
            }
        }
    };
}

pub trait AsElements {
    fn concrete_element(&self) -> Elements;
    fn concrete_element_mut(&mut self) -> ElementsMut;
}

pub trait AsTypes {
    fn concrete_type(&self) -> Types;
    fn concrete_type_mut(&mut self) -> TypesMut;
}

generate_definition_wrapper!(
    Module, Struct, Class, Exception, Interface, Enum, TypeAlias
);

generate_elements_wrapper!(
    Module, Struct, Class, Exception, DataMember, Interface, Operation, Parameter, Enum,
    Enumerator, TypeAlias, TypeRef, Sequence, Dictionary, Primitive, Identifier, Attribute
);

generate_types_wrapper!(
    Struct, Class, Interface, Enum, Sequence, Dictionary, Primitive
);

generate_typerefs_wrapper!(
    Struct, Class, Exception, Interface, Enum, Sequence, Dictionary, Primitive
);

implement_as_elements!(Module);
implement_as_types!(Struct);
implement_as_types!(Class);
implement_as_elements!(Exception);
implement_as_elements!(DataMember);
implement_as_types!(Interface);
implement_as_elements!(Operation);
implement_as_elements!(Parameter);
implement_as_types!(Enum);
implement_as_elements!(Enumerator);
implement_as_elements!(TypeAlias); //TODO rethink type-aliases.
implement_as_types!(Sequence);
implement_as_types!(Dictionary);
implement_as_types!(Primitive);
implement_as_elements!(Identifier);
implement_as_elements!(Attribute);

// Since `TypeRef` has a generic type parameter, we implement the as_wrapper methods by hand.
impl<T: Element + ?Sized> AsElements for TypeRef<T> {
    fn concrete_element(&self) -> Elements {
        self.definition().concrete_element()
    }

    fn concrete_element_mut(&mut self) -> ElementsMut {
        panic!() // TODO write a message here!
    }
}

impl<T: Type + ?Sized> AsTypes for TypeRef<T> {
    fn concrete_type(&self) -> Types {
        self.definition().concrete_type()
    }

    fn concrete_type_mut(&mut self) -> TypesMut {
        panic!() // TODO write a message here!
    }
}
