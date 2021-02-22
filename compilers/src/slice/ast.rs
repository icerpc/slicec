
use crate::grammar::*;
use crate::visitor::Visitable;

//------------------------------------------------------------------------------
// Definition
//------------------------------------------------------------------------------
pub trait Definition : Element + Visitable {
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
