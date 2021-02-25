
use crate::grammar::*;
use crate::visitor::{Visitable, Visitor};

//------------------------------------------------------------------------------
// Node
//------------------------------------------------------------------------------
/// Nodes represent (and own) grammar elements that can be referenced by the compiler or other grammar elements.
///
/// Elements are wrapped in a Node and inserted into the AST vector after creation. They can then be referenced by
/// their index in the AST vector and resolved with the [resolve_index](SliceAst::resolve_index) method.
#[derive(Clone, Debug)]
pub(crate) enum Node {
    Module(Module),
    Struct(Struct),
    Interface(Interface),
    DataMember(DataMember),
}

impl Node {
    /// Sets this node's index by setting it on the underlying grammar element.
    ///
    /// Grammar elements that can be a node have a default index of `usize::Max` set during construction.
    /// The actual index is only set when the node is placed into the AST vector.
    /// This is the only time this method should be called. Invoking it again will cause a panic.
    pub(crate) fn set_index(&mut self, index: usize) {
        // Get a reference to the underlying element's `index` field.
        let old_index = match self {
            Self::Module(module_def)       => { &mut module_def.index },
            Self::Struct(struct_def)       => { &mut struct_def.index },
            Self::Interface(interface_def) => { &mut interface_def.index },
            Self::DataMember(data_member)  => { &mut data_member.index },
        };

        // Ensure the index hasn't already been set.
        debug_assert!(*old_index == usize::MAX, "Node index has already been set!\n{:?}", self);

        // Set the new index.
        *old_index = index;
    }

    /// Returns this node's index from the underlying grammar element.
    pub(crate) fn get_index(&self) -> usize {
        match self {
            Self::Module(module_def)       => { module_def.index },
            Self::Struct(struct_def)       => { struct_def.index },
            Self::Interface(interface_def) => { interface_def.index },
            Self::DataMember(data_member)  => { data_member.index },
        }
    }
}

impl Visitable for Node {
    fn visit(&self, visitor: &mut dyn Visitor, ast: &SliceAst) {
        // Forward the `visit` call to the underlying element.
        match self {
            Self::Module(module_def)       => { module_def.visit(visitor, ast) },
            Self::Struct(struct_def)       => { struct_def.visit(visitor, ast) },
            Self::Interface(interface_def) => { interface_def.visit(visitor, ast) },
            Self::DataMember(data_member)  => { data_member.visit(visitor, ast) },
        }
    }
}

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
#[derive(Debug)]
pub struct SliceAst {
    /// The AST vector where all the 'tree' nodes are stored, in the order the parser parsed them.
    ast: Vec<Node>,
}

impl SliceAst {
    /// Creates an empty AST with no entries in it.
    pub(crate) fn new() -> Self {
        SliceAst { ast: Vec::new() }
    }

    /// Moves the provided node into the AST vector and sets the node's index.
    pub(crate) fn add_node(&mut self, mut node: Node) -> usize {
        let index = self.ast.len();
        node.set_index(index);

        self.ast.push(node);
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
