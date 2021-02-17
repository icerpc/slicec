
use crate::parser::SliceAst;
use crate::grammar::*;
use crate::visitor::Visitor;

use std::collections::HashMap;

//------------------------------------------------------------------------------
// LookupTableBuilder
//------------------------------------------------------------------------------
pub struct LookupTableBuilder {
    lookup_table: HashMap<String, usize>,
    type_indices: Vec<usize>,
    current_scope: Vec<String>,
}

impl LookupTableBuilder {
    pub fn new() -> Self {
        LookupTableBuilder {
            lookup_table: HashMap::new(),
            type_indices: Vec::new(),
            current_scope: Vec::new(),
        }
    }

    pub fn into_tables(self) -> (HashMap<String, usize>, Vec<usize>) {
        (self.lookup_table, self.type_indices)
    }

    fn scoped_identifier(&self, identifier: &str) -> String {
        if self.current_scope.len() > 0 {
            self.current_scope.join("::") + "::" + identifier
        } else {
            identifier.to_owned()
        }
    }
}

impl Visitor for LookupTableBuilder {
    fn visit_module_start(&mut self, module_def: &mut Module, _: &mut SliceAst) {
        let identifier = module_def.get_identifier();
        self.lookup_table.insert(self.scoped_identifier(identifier), module_def.def_index);

        self.current_scope.push(identifier.to_owned());
    }

    fn visit_module_end(&mut self, _: &mut Module, _: &mut SliceAst) {
        self.current_scope.pop();
    }

    fn visit_struct_start(&mut self, struct_def: &mut Struct, _: &mut SliceAst) {
        let scoped_identifier = self.scoped_identifier(struct_def.get_identifier());
        self.lookup_table.insert(scoped_identifier, struct_def.def_index);
        self.type_indices.push(struct_def.def_index);
    }

    fn visit_interface_start(&mut self, interface_def: &mut Interface, _: &mut SliceAst) {
        let scoped_identifier = self.scoped_identifier(interface_def.get_identifier());
        self.lookup_table.insert(scoped_identifier, interface_def.def_index);
        self.type_indices.push(interface_def.def_index);
    }
}

//------------------------------------------------------------------------------
// TreePatcher
//------------------------------------------------------------------------------
pub struct TreePatcher<'a> {
    lookup_table: &'a HashMap<String, usize>,
    type_indices: &'a Vec<usize>,
}

impl<'a> TreePatcher<'a> {
    pub fn new(lookup_table: &'a HashMap<String, usize>, type_indices: &'a Vec<usize>) -> Self{
        TreePatcher { lookup_table, type_indices }
    }
}

impl<'a> Visitor for TreePatcher<'a> {
    
}
