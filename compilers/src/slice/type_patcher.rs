
use crate::grammar::*;
use crate::parser::Definition;
use crate::visitor::Visitor;

use std::collections::HashMap;

//------------------------------------------------------------------------------
// LookupTableBuilder
//------------------------------------------------------------------------------
pub struct LookupTableBuilder<'a> {
    definition_table: HashMap<String, usize>,
    type_table: HashMap<String, usize>,
    current_scope: Vec<String>,
    ast: &'a mut Vec<Box<dyn Definition>>,
}

impl<'a> LookupTableBuilder<'a> {
    pub fn new() -> Self {
        LookupTableBuilder {
            type_table: HashMap::new(),
            definition_table: HashMap::new(),
        }
    }

    pub fn get_tables(self) -> (HashMap<String, usize>, HashMap<String, usize>) {
        (self.definition_table, self.type_table)
    }
}

impl<'a> Visitor for LookupTableBuilder<'a> {
    fn visit_module(&mut self, module_def: &mut Module) {

    }

    fn visit_struct(&mut self, struct_def: &mut Struct) {

    }

    fn visit_interface(&mut self, interface_def: &mut Interface) {

    }

    fn visit_data_member(&mut self, data_member: &mut DataMember) {

    }

    fn visit_identifier(&mut self, identifier: &mut Identifier) {

    }

    fn visit_type_use(&mut self, type_use: &mut TypeUse) {

    }

    fn resolve_id(&mut self, id: usize) -> &mut dyn Node {

    }
}

//------------------------------------------------------------------------------
// TypePatcher
//------------------------------------------------------------------------------
pub struct TypePatcher<'a> {

}

impl<'a> TypePatcher<'a> {

}

impl<'a> Visitor for TypePatcher<'a> {
    fn visit_module(&mut self, module_def: &mut Module) {

    }

    fn visit_struct(&mut self, struct_def: &mut Struct) {

    }

    fn visit_interface(&mut self, interface_def: &mut Interface) {

    }

    fn visit_data_member(&mut self, data_member: &mut DataMember) {

    }

    fn visit_identifier(&mut self, identifier: &mut Identifier) {

    }

    fn visit_type_use(&mut self, type_use: &mut TypeUse) {

    }

    fn resolve_id(&self, id: usize) -> &mut dyn Node {

    }
}
