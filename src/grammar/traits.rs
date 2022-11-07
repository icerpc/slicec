// Copyright (c) ZeroC, Inc. All rights reserved.

use super::comments::DocComment;
use super::elements::{Attribute, Identifier, TypeRef};
use super::util::{Scope, TagFormat};
use super::wrappers::AsTypes;
use super::AttributeKind;
use crate::grammar::attributes;
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

pub trait NamedSymbol: ScopedSymbol {
    fn identifier(&self) -> &str;
    fn raw_identifier(&self) -> &Identifier;

    fn module_scoped_identifier(&self) -> String {
        let module_scope = self.module_scope().to_owned();
        if module_scope.is_empty() {
            self.identifier().to_owned()
        } else {
            module_scope + "::" + self.identifier()
        }
    }

    fn parser_scoped_identifier(&self) -> String {
        let parser_scope = self.parser_scope().to_owned();
        if parser_scope.is_empty() {
            self.identifier().to_owned()
        } else {
            parser_scope + "::" + self.identifier()
        }
    }
}

pub trait Attributable {
    fn attributes(&self) -> &Vec<Attribute>;

    fn has_attribute(&self, directive: &str, recurse: bool) -> bool {
        self.get_raw_attribute(directive, recurse).is_some()
    }

    fn get_attribute(&self, directive: &str, recurse: bool) -> Option<&AttributeKind> {
        self.get_raw_attribute(directive, recurse)
            .map(|attribute| &attribute.kind)
    }

    fn get_attribute_list(&self, directive: &str) -> Vec<Option<&AttributeKind>>;

    fn get_raw_attribute(&self, directive: &str, recurse: bool) -> Option<&Attribute>;

    fn get_ignored_warnings(&self, check_parent: bool) -> Option<&AttributeKind> {
        self.get_attribute(attributes::IGNORE_WARNINGS, check_parent)
    }
}

pub trait Commentable {
    fn comment(&self) -> Option<&DocComment>;
}

pub trait Entity: NamedSymbol + Attributable + Commentable {
    fn get_deprecation(&self, check_parent: bool) -> Option<Option<&String>> {
        match self.get_attribute(attributes::DEPRECATED, check_parent) {
            Some(AttributeKind::Deprecated { reason }) => Some(reason.as_ref()),
            _ => None,
        }
    }
}

pub trait Container<T>: Entity {
    fn contents(&self) -> &Vec<T>;
}

pub trait Contained<T: Entity + ?Sized>: Entity {
    fn parent(&self) -> Option<&T>;
}

pub trait Member: Entity {
    fn data_type(&self) -> &TypeRef;
    fn tag(&self) -> Option<u32>;

    fn is_tagged(&self) -> bool {
        self.tag().is_some()
    }
}

pub trait Type: Element + AsTypes {
    fn type_string(&self) -> String;
    fn is_fixed_size(&self) -> bool;
    fn min_wire_size(&self) -> u32;
    fn uses_classes(&self) -> bool;
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
        }
    };
}

macro_rules! implement_Attributable_for {
    ($type:ty) => {
        impl Attributable for $type {
            fn attributes(&self) -> &Vec<Attribute> {
                &self.attributes
            }

            fn get_attribute_list(&self, directive: &str) -> Vec<Option<&AttributeKind>> {
                let mut result = vec![self.get_attribute(directive, false)];

                if let Some(parent) = self.parent() {
                    result.extend(parent.get_attribute_list(directive))
                }

                result
            }

            fn get_raw_attribute(&self, directive: &str, recurse: bool) -> Option<&Attribute> {
                for attribute in &self.attributes {
                    if attribute.directive() == directive {
                        return Some(attribute);
                    }
                }

                match self.parent() {
                    Some(parent) if recurse => parent.get_raw_attribute(directive, recurse),
                    _ => None,
                }
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
        implement_Commentable_for!($type);

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
            fn parent(&self) -> Option<&$container_type> {
                Some(self.parent.borrow())
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

            fn tag(&self) -> Option<u32> {
                self.tag // Return by copy
            }
        }
    };
}

pub(crate) use {
    implement_Attributable_for, implement_Commentable_for, implement_Contained_for, implement_Container_for,
    implement_Element_for, implement_Entity_for, implement_Member_for, implement_Named_Symbol_for,
    implement_Scoped_Symbol_for, implement_Symbol_for,
};
