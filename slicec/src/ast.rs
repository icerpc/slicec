// Copyright (c) ZeroC, Inc. All rights reserved.

use crate::grammar::*;
use std::collections::HashMap;

/// Nodes own and represent grammar elements that can be referenced by other elements or the parser.
///
/// Elements are wrapped in a Node and inserted into the AST vector after creation. They can then be
/// referenced by their index in the AST and resolved with [resolve_index](Ast::resolve_index).
#[derive(Debug)]
pub enum Node {
    Module(usize, Module),
    Struct(usize, Struct),
    Interface(usize, Interface),
    Enum(usize, Enum),
    Operation(usize, Operation),
    DataMember(usize, DataMember),
    Parameter(usize, Parameter),
    Enumerator(usize, Enumerator),
    Sequence(usize, Sequence),
    Dictionary(usize, Dictionary),
    Primitive(usize, Primitive),
}

impl Node {
    /// Unwraps a node into an Element and returns a reference to it.
    pub fn as_element(&self) -> &dyn Element {
        match self {
            Self::Module(_, module_def)       => module_def,
            Self::Struct(_, struct_def)       => struct_def,
            Self::Interface(_, interface_def) => interface_def,
            Self::Enum(_, enum_def)           => enum_def,
            Self::Operation(_, operation)     => operation,
            Self::DataMember(_, data_member)  => data_member,
            Self::Parameter(_, parameter)     => parameter,
            Self::Enumerator(_, enumerator)   => enumerator,
            Self::Sequence(_, sequence)       => sequence,
            Self::Dictionary(_, dictionary)   => dictionary,
            Self::Primitive(_, primitive)     => primitive,
        }
    }

    /// Unwraps a node if it contains a struct implementing `NamedSymbol` and returns a reference to
    /// it. If the underlying struct doesn't implement it, this returns `None`.
    pub fn as_named_symbol(&self) -> Option<&dyn NamedSymbol> {
        match self {
            Self::Module(_, module_def)       => Some(module_def),
            Self::Struct(_, struct_def)       => Some(struct_def),
            Self::Interface(_, interface_def) => Some(interface_def),
            Self::Enum(_, enum_def)           => Some(enum_def),
            Self::Operation(_, operation)     => Some(operation),
            Self::DataMember(_, data_member)  => Some(data_member),
            Self::Parameter(_, parameter)     => Some(parameter),
            Self::Enumerator(_, enumerator)   => Some(enumerator),
            _ => None,
        }
    }

    /// Unwraps a node if it contains a struct implementing `Type` and returns a reference to it.
    /// If the underlying struct doesn't implement it, this returns `None`.
    pub fn as_type(&self) -> Option<&dyn Type> {
        match self {
            Self::Struct(_, struct_def)       => Some(struct_def),
            Self::Interface(_, interface_def) => Some(interface_def),
            Self::Enum(_, enum_def)           => Some(enum_def),
            Self::Sequence(_, sequence)       => Some(sequence),
            Self::Dictionary(_, dictionary)   => Some(dictionary),
            Self::Primitive(_, primitive)     => Some(primitive),
            _ => None,
        }
    }
}

/// Attempts to unwrap a node to a specified underlying type. If the node is the specified type,
/// it is unwrapped, and a reference is returned to the underlying element.
/// Otherwise this panics.
#[macro_export]
macro_rules! ref_from_node {
    ($a:path, $b:expr, $c:expr) => {{
        let resolved = $b.resolve_index($c);
        if let $a(_, element) = resolved {
            element
        } else {
            panic!(
                "Node #{} contains a {} when a {} was expected!",
                $c,
                resolved.as_element().kind(),
                stringify!($a),
            );
        }
    }};
}

/// Attempts to unwrap a node to a specified underlying type. If the node is the specified type,
/// it is unwrapped, and a mutable reference is returned to the underlying element.
/// Otherwise this panics.
#[macro_export]
macro_rules! mut_ref_from_node {
    ($a:path, $b:expr, $c:expr) => {{
        let resolved = $b.resolve_index_mut($c);
        if let $a(_, element) = resolved {
            element
        } else {
            panic!(
                "Node #{} contains a {} when a {} was expected!",
                $c,
                resolved.as_element().kind(),
                stringify!($a),
            );
        }
    }};
}

/// This trait provides a conversion method to simplify wrapping an element in a node.
/// Only elements implementing this trait can be stored in nodes, and hence the AST vector.
pub(crate) trait IntoNode {
    /// Wraps `self` into a node that stores it and it's index in the AST.
    fn into_node(self, index: usize) -> Node;
}

/// This macro implements the `IntoNode` trait for an element, to reduce repetitive implementations.
macro_rules! implement_into_node_for {
    ($a:ty, $b:path) => {
        impl IntoNode for $a {
            fn into_node(self, index: usize) -> Node {
                $b(index, self)
            }
        }
    };
}

implement_into_node_for!(Module, Node::Module);
implement_into_node_for!(Struct, Node::Struct);
implement_into_node_for!(Interface, Node::Interface);
implement_into_node_for!(Enum, Node::Enum);
implement_into_node_for!(Operation, Node::Operation);
implement_into_node_for!(DataMember, Node::DataMember);
implement_into_node_for!(Parameter, Node::Parameter);
implement_into_node_for!(Enumerator, Node::Enumerator);
implement_into_node_for!(Sequence, Node::Sequence);
implement_into_node_for!(Dictionary, Node::Dictionary);
implement_into_node_for!(Primitive, Node::Primitive);

/// The Abstract Syntax Tree is where all slice grammar elements are stored, directly or indirectly.
///
/// All elements parsed by the compiler are stored in a single instance of Ast, even those from
/// different slice files. Hence there is no notion of 'file' at the semantic level, all elements
/// are only grouped by module. Storing all elements in a single common AST also simplifies
/// cross-file referencing. All definitions are referencable from all slice files, without needing
/// to explicitely include or import files by name.
///
/// Internally the AST is implemented as a 'flattened' vector of nodes, instead of a literal tree
/// structure. This simplifies storage and memory layout, and allows nodes to be referenced by their
/// index in the vector, instead of needing an actual memory reference. This is especially important
/// in Rust where references are strictly managed. Additionally, it simplifies ownership semantics,
/// since all nodes are directly owned by the vector, instead of having parents that own their
/// children, like normal trees do.
#[derive(Debug, Default)]
pub struct Ast {
    /// The AST vector where all the nodes are stored, in the order the parser parsed them.
    ast: Vec<Node>,
    /// Cache of all the primitives that have been added to the AST, and their indexes in it.
    /// This allows only one copy of each primitive to be needed in the AST, instead of having
    /// excessive copies of primitives every time they're used.
    primitive_cache: HashMap<Primitive, usize>,
}

impl Ast {
    /// Retrieves an immutable reference to the node at the specified index.
    /// # Panics
    /// This method panics if `index` doesn't represent a valid node.
    pub fn resolve_index(&self, index: usize) -> &Node {
        &self.ast[index]
    }

    /// Retrieves a mutable reference to the node at the specified index.
    /// # Panics
    /// This method panics if `index` doesn't represent a valid node.
    pub(crate) fn resolve_index_mut(&mut self, index: usize) -> &mut Node {
        &mut self.ast[index]
    }

    /// Returns an iterator that traverses the nodes in the tree in parse order.
    /// Each node accessed by the iterator is presented as a mutable reference.
    pub(crate) fn iter_mut(&mut self) -> std::slice::IterMut<'_, Node> {
        self.ast.iter_mut()
    }

    /// Wraps the provided element in a Node and moves it into the AST vector.
    pub(crate) fn add_element(&mut self, element: impl IntoNode) -> usize {
        let index = self.ast.len();
        self.ast.push(element.into_node(index));
        index
    }

    /// Wraps the provided Primitive in a node and moves it into the AST vector.
    /// This method caches and returns it's index. If the primitive was already added to the AST
    /// instead of re-adding it, the value is dropped and it's cached index is returned instead.
    /// This prevents excessive copies of primitives being in the AST, when they're all identical.
    pub(crate) fn add_primitive(&mut self, primitive: Primitive) -> usize {
        match self.primitive_cache.get(&primitive) {
            Some(index) => *index,
            None => {
                // Add the primitive into the AST and cache it's index.
                let index = self.add_element(primitive);
                self.primitive_cache.insert(primitive, index);
                index
            }
        }
    }
}
