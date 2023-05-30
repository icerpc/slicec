// Copyright (c) ZeroC, Inc.

use super::comments::DocComment;
use super::elements::{Attribute, Identifier, Integer, TypeRef};
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
    fn module_scope(&self) -> &str;
    fn parser_scope(&self) -> &str;
    fn raw_scope(&self) -> &Scope;
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

    /// Returns true if the predicate matches any attribute. False otherwise.
    fn has_attribute<P, T>(&self, predicate: P) -> bool
    where
        Self: Sized,
        P: FnMut(&Attribute) -> Option<T>,
    {
        self.find_attribute(predicate).is_some()
    }

    /// Returns the first attribute that matches the predicate.
    fn find_attribute<P, T>(&self, predicate: P) -> Option<T>
    where
        Self: Sized,
        P: FnMut(&Attribute) -> Option<T>,
    {
        self.attributes().into_iter().find_map(predicate)
    }
}

pub trait Entity: ScopedSymbol + NamedSymbol + Attributable + AsEntities {
    fn get_deprecation(&self) -> Option<Option<String>> {
        self.attributes().into_iter().find_map(Attribute::match_deprecated)
    }
}

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
            fn module_scope(&self) -> &str {
                &self.scope.raw_module_scope
            }

            fn parser_scope(&self) -> &str {
                &self.scope.raw_parser_scope
            }

            fn raw_scope(&self) -> &Scope {
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
    ($type:ty) => {
        impl Attributable for $type {
            fn attributes(&self) -> Vec<&Attribute> {
                self.attributes.iter().map(WeakPtr::borrow).collect::<Vec<_>>()
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
        implement_Named_Symbol_for!($type);
        implement_Scoped_Symbol_for!($type);
        implement_Attributable_for!($type);

        impl Entity for $type {}
    };
}

macro_rules! implement_Container_for {
    ($type:ty, $contained_type:ty, $field_name:ident) => {
        impl Container<$contained_type> for $type {
            fn contents(&self) -> &Vec<$contained_type> {
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
