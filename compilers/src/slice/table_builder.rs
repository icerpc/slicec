
use crate::ast::SliceAst;
use crate::grammar::*;
use crate::visitor::Visitor;

use std::collections::HashMap;

//TODO: We should build more tables instead of just the type-table.

//------------------------------------------------------------------------------
// TableBuilder
//------------------------------------------------------------------------------
pub struct TableBuilder {
    type_table: HashMap<String, usize>,
    current_scope: Vec<String>,
}

impl TableBuilder {
    pub fn new() -> Self {
        TableBuilder {
            type_table: HashMap::new(),
            current_scope: Vec::new(),
        }
    }

    pub fn into_table(self) -> HashMap<String, usize> {
        self.type_table
    }
}

impl Visitor for TableBuilder {
    fn visit_module_start(&mut self, module_def: &Module) {
        self.current_scope.push(module_def.get_identifier().to_owned());
    }

    fn visit_module_end(&mut self, _: &Module) {
        self.current_scope.pop();
    }

    fn visit_struct_start(&mut self, struct_def: &Struct) {
        let scoped_name = self.current_scope.join("::") + "::" + struct_def.get_identifier();
        self.type_table.insert(scoped_name, struct_def.def_index);
    }

    fn visit_interface_start(&mut self, interface_def: &Interface) {
        let scoped_name = self.current_scope.join("::") + "::" + interface_def.get_identifier();
        self.type_table.insert(scoped_name, interface_def.def_index);
    }
}
