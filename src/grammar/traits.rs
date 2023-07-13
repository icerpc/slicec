// Copyright (c) ZeroC, Inc.

use super::attributes::AttributeKind;
use super::comments::DocComment;
use super::elements::{Attribute, Identifier, Integer, Module, TypeRef};
use super::util::{Scope, TagFormat};
use super::wrappers::{AsEntities, AsTypes};
use crate::slice_file::Span;
use crate::supported_encodings::SupportedEncodings;

pub trait Element: std::fmt::Debug {
    fn kind(&self) -> &'static str;
}

pub trait Symbol: Element {
    fn span(&self) -> &Span;
}

pub trait ScopedSymbol: Symbol {
    fn parser_scope(&self) -> &str;
    fn module_scope(&self) -> &str;
    fn get_module(&self) -> &Module;
    fn get_raw_scope(&self) -> &Scope;
}

pub trait NamedSymbol: Symbol {
    fn identifier(&self) -> &str;
    fn raw_identifier(&self) -> &Identifier;
    fn module_scoped_identifier(&self) -> String;
    fn parser_scoped_identifier(&self) -> String;
}

pub trait Attributable {
    /// Returns the attributes of the element.
    fn attributes(&self) -> Vec<&Attribute>;

    /// Returns all the attributes of the element and its parents.
    fn all_attributes(&self) -> Vec<Vec<&Attribute>>;
}

// These functions are declared in a separate trait because they have type parameters, making them not 'object-safe'.
// This restricts how you're allowed to store and pass around dynamically typed instances of the trait.
//
// By having them in a separate trait, this allows the main `Attributable` trait to be free of restrictions, while still
// having access to all these functions (because of the blanket impl underneath this trait definition).
pub trait AttributeFunctions {
    /// Returns true if this element has an attribute of the specified type and false otherwise.
    fn has_attribute<T: AttributeKind + 'static>(&self) -> bool;

    /// Returns the first attribute of the specified type that is applied to this element.
    /// If no attributes of the specified type can be found, this returns `None`.
    fn find_attribute<T: AttributeKind + 'static>(&self) -> Option<&T>;

    /// Returns all the attributes applied to this element that are of the specified type.
    fn find_attributes<T: AttributeKind + 'static>(&self) -> Vec<&T>;
}

// Blanket impl to ensure that everything implementing `Attributable` also gets `AttributeFunctions` for free.
impl<A: Attributable + ?Sized> AttributeFunctions for A {
    fn has_attribute<T: AttributeKind + 'static>(&self) -> bool {
        self.find_attribute::<T>().is_some()
    }

    fn find_attribute<T: AttributeKind + 'static>(&self) -> Option<&T> {
        self.attributes().into_iter().find_map(Attribute::downcast)
    }

    fn find_attributes<T: AttributeKind + 'static>(&self) -> Vec<&T> {
        self.attributes().into_iter().filter_map(Attribute::downcast).collect()
    }
}

pub trait Entity: ScopedSymbol + NamedSymbol + Attributable + AsEntities {}

pub trait Container<T>: Entity {
    fn contents(&self) -> &Vec<T>;
}

pub trait Contained<T: Entity + ?Sized>: Entity {
    fn parent(&self) -> &T;
}

pub trait Member: Entity {
    fn data_type(&self) -> &TypeRef;
    fn raw_tag(&self) -> Option<&Integer<u32>>;

    fn tag(&self) -> Option<u32> {
        self.raw_tag().map(|tag| tag.value)
    }

    fn is_tagged(&self) -> bool {
        self.raw_tag().is_some()
    }
}

pub trait Commentable: Entity {
    fn comment(&self) -> Option<&DocComment>;
}

pub trait Type: Element + AsTypes {
    fn type_string(&self) -> String;
    fn fixed_wire_size(&self) -> Option<u32>;
    fn is_class_type(&self) -> bool;
    fn tag_format(&self) -> Option<TagFormat>;
    fn supported_encodings(&self) -> SupportedEncodings;
}

macro_rules! implement_Element_for {
    ($type:ty, $kind_string:literal$(, $($bounds:tt)+)?) => {
        impl$(<T: $($bounds)+>)? Element for $type {
            fn kind(&self) -> &'static str {
                $kind_string
            }
        }
    };
}

macro_rules! implement_Symbol_for {
    ($type:ty$(, $($bounds:tt)+)?) => {
        impl$(<T: $($bounds)+>)? Symbol for $type {
            fn span(&self) -> &Span {
                &self.span
            }
        }
    };
}

macro_rules! implement_Scoped_Symbol_for {
    ($type:ty$(, $($bounds:tt)+)?) => {
        impl$(<T: $($bounds)+>)? ScopedSymbol for $type {
            fn parser_scope(&self) -> &str {
                &self.scope.parser_scope
            }

            fn module_scope(&self) -> &str {
                match &self.scope.module {
                    Some(module_ptr) => module_ptr.borrow().nested_module_identifier(),
                    None => "",
                }
            }

            fn get_module(&self) -> &Module {
                self.scope.module.as_ref().unwrap().borrow()
            }

            fn get_raw_scope(&self) -> &Scope {
                &self.scope
            }
        }
    };
}

macro_rules! implement_Named_Symbol_for {
    ($type:ty) => {
        impl NamedSymbol for $type {
            fn identifier(&self) -> &str {
                &self.identifier.value
            }

            fn raw_identifier(&self) -> &Identifier {
                &self.identifier
            }

            fn module_scoped_identifier(&self) -> String {
                util::get_scoped_identifier(self.identifier(), self.module_scope())
            }

            fn parser_scoped_identifier(&self) -> String {
                util::get_scoped_identifier(self.identifier(), self.parser_scope())
            }
        }
    };
}

macro_rules! implement_Attributable_for {
    ($type:ty$(, $($bounds:tt)+)?) => {
        impl$(<T: $($bounds)+>)? Attributable for $type {
            fn attributes(&self) -> Vec<&Attribute> {
                self.attributes.iter().map(WeakPtr::borrow).collect()
            }

            fn all_attributes(&self) -> Vec<Vec<&Attribute>> {
                vec![self.attributes()]
            }
        }
    };
    (@Contained $type:ty$(, $($bounds:tt)+)?) => {
        impl$(<T: $($bounds)+>)? Attributable for $type {
            fn attributes(&self) -> Vec<&Attribute> {
                self.attributes.iter().map(WeakPtr::borrow).collect()
            }

            fn all_attributes(&self) -> Vec<Vec<&Attribute>> {
                let mut attributes_list = vec![self.attributes()];
                attributes_list.extend(self.parent().all_attributes());
                attributes_list
            }
        }
    };
}

macro_rules! implement_Commentable_for {
    ($type:ty) => {
        impl Commentable for $type {
            fn comment(&self) -> Option<&DocComment> {
                self.comment.as_ref()
            }
        }
    };
}

macro_rules! implement_Entity_for {
    ($type:ty) => {
        implement_Symbol_for!($type);
        implement_Scoped_Symbol_for!($type);
        implement_Named_Symbol_for!($type);

        impl Entity for $type {}
    };
}

macro_rules! implement_Container_for {
    ($type:ty, $contained_type:ty, $field_name:ident) => {
        impl Container<WeakPtr<$contained_type>> for $type {
            fn contents(&self) -> &Vec<WeakPtr<$contained_type>> {
                &self.$field_name
            }
        }
    };
}

macro_rules! implement_Contained_for {
    ($type:ty, $container_type:ty) => {
        impl Contained<$container_type> for $type {
            fn parent(&self) -> &$container_type {
                self.parent.borrow()
            }
        }
    };
}

macro_rules! implement_Member_for {
    ($type:ty) => {
        impl Member for $type {
            fn data_type(&self) -> &TypeRef {
                &self.data_type
            }

            fn raw_tag(&self) -> Option<&Integer<u32>> {
                self.tag.as_ref()
            }
        }
    };
}

pub(crate) use {
    implement_Attributable_for, implement_Commentable_for, implement_Contained_for, implement_Container_for,
    implement_Element_for, implement_Entity_for, implement_Member_for, implement_Named_Symbol_for,
    implement_Scoped_Symbol_for, implement_Symbol_for,
};
