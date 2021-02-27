
use crate::grammar::*;

//------------------------------------------------------------------------------
// Node
//------------------------------------------------------------------------------
/// Nodes represent (and own) grammar elements that can be referenced by the compiler or other grammar elements.
///
/// Elements are wrapped in a Node and inserted into the AST vector after creation. They can then be referenced by
/// their index in the AST vector and resolved with the [resolve_index](SliceAst::resolve_index) method.
#[derive(Clone, Debug)]
pub(crate) enum Node {
    Module(usize, Module),
    Struct(usize, Struct),
    Interface(usize, Interface),
    DataMember(usize, DataMember),
}

impl Node {
    /// Returns this node's index.
    pub(crate) fn index(&self) -> usize {
        match self {
            Self::Module(index, _)     => index,
            Self::Struct(index, _)     => index,
            Self::Interface(index, _)  => index,
            Self::DataMember(index, _) => index,
        }.clone()
    }
}

//------------------------------------------------------------------------------
// IntoNode
//------------------------------------------------------------------------------
/// This trait provides a conversion method to simplify wrapping a grammar element in a node.
/// Only types implementing this trait can be stored in nodes, and hence the AST vector.
pub(crate) trait IntoNode : Element {
    /// Converts an element into a node that contains the element and the node's index in the AST vector.
    fn into_node(self, index: usize) -> Node;
}

/// This macro implements the `IntoTrait` trait for a grammar element, to reduce boilerplate implementations.
macro_rules! implement_into_node_for{
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

//------------------------------------------------------------------------------
// SliceAst
//------------------------------------------------------------------------------
/// SliceAst stores the Abstract Syntax Tree where all slice grammar elements are stored directly, or indirectly.
///
/// All elements parsed by the compiler are stored in a single instance of SliceAst, even those from different slice
/// files. Hence there is no notion of 'file' at the semantic level, all code is only grouped by module.
/// Storing all elements in a single common AST also simplifies cross-file referencing. All definitions are referencable
/// from all slice files, without needing to explicitely include or import the file by name.
///
/// Internally the AST is implemented as a 'flattened' vector of nodes, instead of a literal tree.
/// This simplifies storage and memory layout, and allows nodes to be referenced by their index in the vector, instead
/// of needing an actual memory reference. This is especially important in Rust where references are strictly managed.
/// Additionally, it simplifies the ownership semantics, since all nodes are directly owned by the vector, instead of
/// having parent and children ownership semantics like normal trees do.
#[derive(Debug, Default)]
pub struct SliceAst {
    /// The AST vector where all the 'tree' nodes are stored, in the order the parser parsed them.
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
    ///
    /// # Panics
    /// This method panics if `index` doesn't represent a valid node.
    pub(crate) fn resolve_index(&self, index: usize) -> &Node {
        &self.ast[index]
    }

    /// Retrieves a mutable reference to the node at the specified index.
    ///
    /// # Panics
    /// This method panics if `index` doesn't represent a valid node.
    pub(crate) fn resolve_index_mut(&mut self, index: usize) -> &mut Node {
        &mut self.ast[index]
    }
}
