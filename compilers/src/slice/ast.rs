
use crate::grammar::*;
use crate::visitor::Visitable;

//------------------------------------------------------------------------------
// Node
//------------------------------------------------------------------------------
pub trait Node : Element + Visitable {
    fn set_index(&mut self, index: usize);
    fn index(&self) -> usize;
}

macro_rules! implement_node_for{
    ($a:ty, $b:ident) => {
        impl Node for $a {
            fn set_index(&mut self, index: usize) {
                self.$b = index;
            }

            fn index(&self) -> usize {
                self.$b
            }
        }
    }
}

implement_node_for!(Module, def_index);
implement_node_for!(Struct, def_index);
implement_node_for!(Interface, def_index);
implement_node_for!(DataMember, def_index);

//------------------------------------------------------------------------------
// SliceAst
//------------------------------------------------------------------------------
pub struct SliceAst {
    ast: Vec<Box<dyn Node>>,
}

impl SliceAst {
    pub fn new() -> Self {
        SliceAst { ast: Vec::new() }
    }

    pub fn add_node(&mut self, mut node: Box<dyn Node>) -> usize {
        let index = self.ast.len();
        node.set_index(index);

        self.ast.push(node);
        index
    }

    pub fn resolve_id(&self, id: usize) -> &Box<dyn Node> {
        &self.ast[id]
    }

    pub fn reserve(&mut self, additional: usize) {
        &self.ast.reserve(additional);
    }
}

//------------------------------------------------------------------------------
// SliceFile
//------------------------------------------------------------------------------
#[derive(Debug)]
pub struct SliceFile {
    pub filename: String,
    pub raw_text: String,
    pub definitions: Vec<usize>,
    pub is_source: bool,
    line_positions: Vec<usize>,
}

impl SliceFile {
    pub fn new(filename: String, raw_text: String, definitions: Vec<usize>, is_source: bool) -> Self {
        let mut line_positions = Vec::new();
        for (index, character) in raw_text.chars().enumerate() {
            if character == '\n' {
                line_positions.push(index + 1);
            }
        }

        SliceFile { filename, raw_text, definitions, is_source, line_positions }
    }

    // TODO add methods for getting text snippets from the slice file! (for error reporting)
}
