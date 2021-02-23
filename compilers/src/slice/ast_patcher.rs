
use crate::ast::SliceAst;
use crate::grammar::*;
use crate::visitor::Visitor;

//------------------------------------------------------------------------------
// AstPatcher
//------------------------------------------------------------------------------
pub struct AstPatcher {
    patched_ast: SliceAst,
}

impl AstPatcher {
    pub fn new(unpatched_ast: &SliceAst) -> Self {
        AstPatcher {
            patched_ast: SliceAst::new(),
        }
    }
}

// TODO in the future we'll patch more than just types. We should also patch parents, and maybe even scoped identifiers.
impl Visitor for AstPatcher {
    fn visit_module_start(&mut self, module_def: &Module) {

    }

    fn visit_struct_start(&mut self, struct_def: &Struct) {

    }

    fn visit_interface_start(&mut self, interface_def: &Interface) {

    }

    fn visit_data_member(&mut self, data_member: &DataMember) {

    }
}
