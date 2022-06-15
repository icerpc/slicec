// Copyright (c) ZeroC, Inc. All rights reserved.

//! TODO write a doc comment for the module.

use crate::downgrade_as;
use crate::grammar::*;
use crate::ptr_util::{OwnedPtr, WeakPtr};
use crate::string_util::prefix_with_article;
use convert_case::{Case, Casing};
use std::convert::TryFrom;
use std::fmt;

// Helper macro for generating `TryFrom` conversion methods to unwrap `Node`s to concrete types,
// when the type of element the Node is holding is known.
macro_rules! generate_try_from_node_impl {
    ($variant:ident, $from_type:ty, $to_type:ty, $convert:path) => {
        impl<'a> TryFrom<$from_type> for $to_type {
            type Error = String;

            /// Attempts to unwrap a node to the specified concrete type.
            ///
            /// If the Slice element held by the node is the specified type, this succeeds,
            /// and returns the unwrapped element in the requested container.
            /// Otherwise this method fails and returns an error message.
            fn try_from(node: $from_type) -> Result<$to_type, Self::Error> {
                if let Node::$variant(x) = node {
                    Ok($convert(x))
                } else {
                    Err(format!(
                        "type mismatch: attempted to unwrap {} from a node holding {}",
                        prefix_with_article(stringify!($variant).to_case(Case::Lower)),
                        prefix_with_article(node.to_string().to_case(Case::Lower)),
                    ))
                }
            }
        }
    };
}

// Helper macro for generating the `Node` enum and its enumerators.
macro_rules! generate_node_enum {
    ($($variant:ident),*) => {
        /// Represents a node in the [Abstract Syntax Tree](AST).
        ///
        /// There is a variant for each kind of Slice element that can be stored in the AST,
        /// each variant holds an instance of its corresponding element wrapped in an [OwnedPtr].
        #[derive(Debug)]
        pub enum Node {
            $($variant(OwnedPtr<$variant>),)*
        }

        impl fmt::Display for Node {
            /// Writes the identifier of the variant to the given formatter (pascal cased).
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let name = match self {
                    $(Node::$variant(_) => stringify!($variant),)*
                };
                write!(f, "{}", name)
            }
        }

        // Generate methods for unwrapping nodes to `&mut OwnedPtr`s.
        $(generate_try_from_node_impl!($variant, &'a mut Node, &'a mut OwnedPtr<$variant>, std::convert::identity);)*

        // Generate methods for unwrapping nodes to `&OwnedPtr`s.
        $(generate_try_from_node_impl!($variant, &'a Node, &'a OwnedPtr<$variant>, std::convert::identity);)*

        // Generate methods for unwrapping nodes to `WeakPtr`s.
        $(generate_try_from_node_impl!($variant, &'a Node, WeakPtr<$variant>, OwnedPtr::<$variant>::downgrade);)*

        // Generate methods for unwrapping nodes to references of elements.
        $(generate_try_from_node_impl!($variant, &'a Node, &'a $variant, OwnedPtr::<$variant>::borrow);)*
    }
}

// generate the `Node` enum with variants for every type allowed to be in the AST.
generate_node_enum! {
    Module, Struct, Class, Exception, DataMember, Interface, Operation, Parameter, Enum,
    Enumerator, Trait, CustomType, TypeAlias, Sequence, Dictionary, Primitive
}

impl<'a> TryFrom<&'a Node> for WeakPtr<dyn Type> {
    type Error = String;

    /// Attempts to unwrap a node to a dynamically typed [WeakPtr] holding a Slice [Type].
    ///
    /// If the Slice element held by the node implements [Type], this succeeds,
    /// otherwise this fails and returns an error message.
    fn try_from(node: &'a Node) -> Result<WeakPtr<dyn Type>, Self::Error> {
        match node {
            Node::Struct(struct_ptr)          => Ok(downgrade_as!(struct_ptr, dyn Type)),
            Node::Class(class_ptr)            => Ok(downgrade_as!(class_ptr, dyn Type)),
            Node::Exception(exception_ptr)    => Ok(downgrade_as!(exception_ptr, dyn Type)),
            Node::Interface(interface_ptr)    => Ok(downgrade_as!(interface_ptr, dyn Type)),
            Node::Enum(enum_ptr)              => Ok(downgrade_as!(enum_ptr, dyn Type)),
            Node::Trait(trait_ptr)            => Ok(downgrade_as!(trait_ptr, dyn Type)),
            Node::CustomType(custom_type_ptr) => Ok(downgrade_as!(custom_type_ptr, dyn Type)),
            Node::TypeAlias(type_alias_ptr)   => Ok(downgrade_as!(type_alias_ptr, dyn Type)),
            Node::Sequence(sequence_ptr)      => Ok(downgrade_as!(sequence_ptr, dyn Type)),
            Node::Dictionary(dictionary_ptr)  => Ok(downgrade_as!(dictionary_ptr, dyn Type)),
            Node::Primitive(primitive_ptr)    => Ok(downgrade_as!(primitive_ptr, dyn Type)),
            _ => Err(format!(
                "type mismatch: attempted to unwrap a node holding {} as an `Type`, but {} doesn't implement `Type`",
                prefix_with_article(node.to_string().to_case(Case::Lower)),
                node.to_string().to_case(Case::Lower),
            )),
        }
    }
}

impl<'a> TryFrom<&'a Node> for &'a dyn Type {
    type Error = String;

    /// Attempts to unwrap a node to a dynamically typed reference of a Slice [Type].
    ///
    /// If the Slice element held by the node implements [Type], this succeeds,
    /// otherwise this fails and returns an error message.
    fn try_from(node: &'a Node) -> Result<&'a dyn Type, Self::Error> {
        match node {
            Node::Struct(struct_ptr)          => Ok(struct_ptr.borrow()),
            Node::Class(class_ptr)            => Ok(class_ptr.borrow()),
            Node::Exception(exception_ptr)    => Ok(exception_ptr.borrow()),
            Node::Interface(interface_ptr)    => Ok(interface_ptr.borrow()),
            Node::Enum(enum_ptr)              => Ok(enum_ptr.borrow()),
            Node::Trait(trait_ptr)            => Ok(trait_ptr.borrow()),
            Node::CustomType(custom_type_ptr) => Ok(custom_type_ptr.borrow()),
            Node::TypeAlias(type_alias_ptr)   => Ok(type_alias_ptr.borrow()),
            Node::Sequence(sequence_ptr)      => Ok(sequence_ptr.borrow()),
            Node::Dictionary(dictionary_ptr)  => Ok(dictionary_ptr.borrow()),
            Node::Primitive(primitive_ptr)    => Ok(primitive_ptr.borrow()),
            _ => Err(format!(
                "type mismatch: attempted to unwrap a node holding {} as an `Type`, but {} doesn't implement `Type`",
                prefix_with_article(node.to_string().to_case(Case::Lower)),
                node.to_string().to_case(Case::Lower),
            )),
        }
    }
}

impl<'a> TryFrom<&'a Node> for &'a dyn Entity {
    type Error = String;

    /// Attempts to unwrap a node to a dynamically typed reference of a Slice [Entity].
    ///
    /// If the Slice element held by the node implements [Entity], this succeeds,
    /// otherwise this fails and returns an error message.
    fn try_from(node: &'a Node) -> Result<&'a dyn Entity, Self::Error> {
        match node {
            Node::Module(module_ptr)          => Ok(module_ptr.borrow()),
            Node::Struct(struct_ptr)          => Ok(struct_ptr.borrow()),
            Node::Class(class_ptr)            => Ok(class_ptr.borrow()),
            Node::Exception(exception_ptr)    => Ok(exception_ptr.borrow()),
            Node::DataMember(data_member_ptr) => Ok(data_member_ptr.borrow()),
            Node::Interface(interface_ptr)    => Ok(interface_ptr.borrow()),
            Node::Operation(operation_ptr)    => Ok(operation_ptr.borrow()),
            Node::Parameter(parameter_ptr)    => Ok(parameter_ptr.borrow()),
            Node::Enum(enum_ptr)              => Ok(enum_ptr.borrow()),
            Node::Enumerator(enumerator_ptr)  => Ok(enumerator_ptr.borrow()),
            Node::Trait(trait_ptr)            => Ok(trait_ptr.borrow()),
            Node::CustomType(custom_type_ptr) => Ok(custom_type_ptr.borrow()),
            Node::TypeAlias(type_alias_ptr)   => Ok(type_alias_ptr.borrow()),
            _ => Err(format!(
                "type mismatch: attempted to unwrap a node holding {} as an `Entity`, but {} doesn't implement `Entity`",
                prefix_with_article(node.to_string().to_case(Case::Lower)),
                node.to_string().to_case(Case::Lower),
            )),
        }
    }
}

// Helper macro for generating `Into<Node>` conversion methods for `OwnedPtr`s of Slice elements.
macro_rules! impl_into_node_for {
    ($variant:ident) => {
        impl Into<Node> for OwnedPtr<$variant> {
            // Macro variables in comments aren't expanded, so instead of writing a doc comment
            // normally, we generate documentation for this function using a `doc` attribute.
            #[doc = concat!(
                "Wraps the OwnedPtr<",
                stringify!($variant),
                "> into a [Node] of the corresponding variant [Node::",
                stringify!($variant),
                "].",
            )]
            fn into(self) -> Node {
                Node::$variant(self)
            }
        }
    };
}

// Implement the `Into<Node>` trait for [OwnedPtr]s of the following types:
impl_into_node_for!(Module);
impl_into_node_for!(Struct);
impl_into_node_for!(Class);
impl_into_node_for!(Exception);
impl_into_node_for!(DataMember);
impl_into_node_for!(Interface);
impl_into_node_for!(Operation);
impl_into_node_for!(Parameter);
impl_into_node_for!(Enum);
impl_into_node_for!(Enumerator);
impl_into_node_for!(Trait);
impl_into_node_for!(CustomType);
impl_into_node_for!(TypeAlias);
impl_into_node_for!(Sequence);
impl_into_node_for!(Dictionary);
// We don't implement it on `Primitive`, because primitive types are baked into the compiler,
// so we don't need conversion methods for wrapping them into `Node`s.
