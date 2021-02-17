
use crate::grammar::*;
use crate::util::Location;
use crate::visitor::Visitable;

//------------------------------------------------------------------------------
// Node
//------------------------------------------------------------------------------
pub trait Node : Visitable {
    fn location(&self) -> &Location;
}

macro_rules! implement_node_for{
    ($a:ty, $b:ident) => {
        impl Node for $a {
            fn location(&self) -> &Location {
                &self.$b
            }
        }
    }
}

implement_node_for!(Module, location);
implement_node_for!(Struct, location);
implement_node_for!(Interface, location);
implement_node_for!(DataMember, location);
implement_node_for!(Identifier, location);
implement_node_for!(TypeUse, location);

//------------------------------------------------------------------------------
// Definition
//------------------------------------------------------------------------------
pub trait Definition : Node {
    fn set_index(&mut self, index: usize);
    fn get_index(&self) -> usize;
}

macro_rules! implement_definition_for{
    ($a:ty, $b:ident) => {
        impl Definition for $a {
            fn set_index(&mut self, index: usize) {
                self.$b = index;
            }

            fn get_index(&self) -> usize {
                self.$b
            }
        }
    }
}

implement_definition_for!(Module, def_index);
implement_definition_for!(Struct, def_index);
implement_definition_for!(Interface, def_index);

//------------------------------------------------------------------------------
// SliceAst
//------------------------------------------------------------------------------
pub struct SliceAst {
    ast: Vec<Box<dyn Definition>>,
}

impl SliceAst {
    pub fn new() -> Self {
        SliceAst { ast: Vec::new() }
    }

    pub fn add_definition(&mut self, mut definition: Box<dyn Definition>) -> usize {
        let index = self.ast.len();
        definition.set_index(index);

        self.ast.push(definition);
        index
    }

    pub fn resolve_id(&self, id: usize) -> &Box<dyn Definition> {
        &self.ast[id]
    }

    pub fn resolve_id_mut(&mut self, id: usize) -> &mut Box<dyn Definition> {
        &mut self.ast[id]
    }
}

//------------------------------------------------------------------------------
// SliceFile
//------------------------------------------------------------------------------
#[derive(Debug)]
pub struct SliceFile {
    pub filename: String,
    pub raw_text: String,
    pub contents: Vec<usize>,
    pub is_source: bool,
}

impl SliceFile {
    pub fn new(filename: String, raw_text: String, contents: Vec<usize>, is_source: bool) -> Self {
        SliceFile { filename, raw_text, contents, is_source }
    }
}
