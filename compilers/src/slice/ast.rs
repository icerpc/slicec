
use crate::grammar::*;

//------------------------------------------------------------------------------
// Node
//------------------------------------------------------------------------------
/// Nodes represent (and own) grammar elements that can be referenced by the compiler or other grammar elements.
///
/// Elements are wrapped in a Node and inserted into the AST vector after creation. They can then be referenced by
/// their index in the AST and resolved with the [resolve_index](SliceAst::resolve_index) method.
#[derive(Clone, Debug)]
pub enum Node {
    Module(usize, Module),
    Struct(usize, Struct),
    Interface(usize, Interface),
    DataMember(usize, DataMember),
    Builtin(usize, Builtin),
}

impl Node {
    /// Unwraps the node if it contains a struct implementing `NamedSymbol` and returns a reference to it.
    /// If the underlying struct doesn't implement it, this returns `None`.
    pub fn as_named_symbol(&self) -> Option<&dyn NamedSymbol> {
        match self {
            Self::Module(_, module_def)       => Some(module_def),
            Self::Struct(_, struct_def)       => Some(struct_def),
            Self::Interface(_, interface_def) => Some(interface_def),
            Self::DataMember(_, data_member)  => Some(data_member),
            _ => None,
        }
    }

    /// Unwraps the node if it contains a struct implementing `Type` and returns a reference to it.
    /// If the underlying struct doesn't implement it, this returns `None`.
    pub fn as_type(&self) -> Option<&dyn Type> {
        match self {
            Self::Struct(_, struct_def)       => Some(struct_def),
            Self::Interface(_, interface_def) => Some(interface_def),
            Self::Builtin(_, builtin)         => Some(builtin),
            _ => None,
        }
    }

    /// Returns the Rust TypeId of the node's underlying type. This function is only enabled in debug configurations.
    #[cfg(debug_assertions)]
    pub(crate) fn type_id(&self) -> std::any::TypeId {
        match self {
            Self::Module(_, _)     => std::any::TypeId::of::<Module>(),
            Self::Struct(_, _)     => std::any::TypeId::of::<Struct>(),
            Self::Interface(_, _)  => std::any::TypeId::of::<Interface>(),
            Self::DataMember(_, _) => std::any::TypeId::of::<DataMember>(),
            Self::Builtin(_, _)    => std::any::TypeId::of::<Builtin>(),
        }.clone()
    }
}

//------------------------------------------------------------------------------
// IntoNode
//------------------------------------------------------------------------------
/// This trait provides a conversion method to simplify wrapping an element in a node.
/// Only elements implementing this trait can be stored in nodes, and hence the AST vector.
pub(crate) trait IntoNode {
    /// Wraps `self` into a node that stores it and it's index in the AST.
    fn into_node(self, index: usize) -> Node;
}

/// This macro implements the `IntoNode` trait for an element, to reduce boilerplate implementations.
macro_rules! implement_into_node_for {
    ($a:ty, $b:path) => {
        impl IntoNode for $a {
            fn into_node(self, index: usize) -> Node {
                $b(index, self)
            }
        }
    }
}

implement_into_node_for!(Module, Node::Module);
implement_into_node_for!(Struct, Node::Struct);
implement_into_node_for!(Interface, Node::Interface);
implement_into_node_for!(DataMember, Node::DataMember);
implement_into_node_for!(Builtin, Node::Builtin);

//------------------------------------------------------------------------------
// SliceAst
//------------------------------------------------------------------------------
/// SliceAst stores the Abstract Syntax Tree where all slice grammar elements are stored, directly or indirectly.
///
/// All elements parsed by the compiler are stored in a single instance of SliceAst, even those from different slice
/// files. Hence there is no notion of 'file' at the semantic level, all elements are only grouped by module.
/// Storing all elements in a single common AST also simplifies cross-file referencing. All definitions are referencable
/// from all slice files, without needing to explicitely include or import files by name.
///
/// Internally the AST is implemented as a 'flattened' vector of nodes, instead of a literal tree structure.
/// This simplifies storage and memory layout, and allows nodes to be referenced by their index in the vector, instead
/// of needing an actual memory reference. This is especially important in Rust where references are strictly managed.
/// Additionally, it simplifies ownership semantics, since all nodes are directly owned by the vector, instead of
/// having parents that own their children, like normal trees do.
#[derive(Debug, Default)]
pub struct SliceAst {
    /// The AST vector where all the nodes are stored, in the order the parser parsed them.
    ast: Vec<Node>,
}

impl SliceAst {
    /// Wraps the provided element in a Node and moves it into the AST vector.
    pub(crate) fn add_element(&mut self, element: impl IntoNode) -> usize {
        let index = self.ast.len();
        self.ast.push(element.into_node(index));
        index
    }

    /// Retrieves an immutable reference to the node at the specified index.
    /// # Panics
    /// This method panics if `index` doesn't represent a valid node.
    pub(crate) fn resolve_index(&self, index: usize) -> &Node {
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
}
