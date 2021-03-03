
use crate::ast::SliceAst;
use crate::error::ErrorHandler;
use crate::grammar::*;
use crate::util::SliceFile;
use crate::visitor::Visitor;
use std::collections::HashMap;

//------------------------------------------------------------------------------
// TableBuilder
//------------------------------------------------------------------------------
/// TableBuilder visits all the named symbols in a set of slice files and generates a lookup table that allows
/// those symbols to be retrieved from the AST by their identifier, instead of requiring an index.
///
/// The table's keys are fully scoped identifiers and it's values are the corresponding symbol's index in the AST.
#[derive(Debug)]
pub(crate) struct TableBuilder<'a> {
    /// This stack holds the identifiers of any enclosing scopes the builder is currently visiting within.
    ///
    /// Except for the 1st element, which is always the empty string.
    /// This represents the global scope, and ensures we always have a leading '::' when joining identifiers together.
    current_scope: Vec<String>,
    /// The lookup table that this struct builds.
    lookup_table: HashMap<String, usize>,
    /// Reference to the compiler's error handler so the builder can output errors.
    error_handler: &'a mut ErrorHandler,
}

impl<'a> TableBuilder<'a> {
    /// Builds a lookup table for all the named symbols defined in the provided slice files.
    /// The table maps a symbol's fully scoped identifier to it's index in the AST.
    pub(crate) fn build_lookup_table(files: &HashMap<String, SliceFile>,
                                     ast: &SliceAst,
                                     error_handler: &'a mut ErrorHandler) -> HashMap<String, usize> {
        let mut table_builder = TableBuilder::new(error_handler);
        for file in files.values() {
            file.visit_with(&mut table_builder, ast);
        }
        table_builder.lookup_table
    }

    /// Creates a new `TableBuilder` with an empty lookup table, and starting at global scope ("::").
    fn new(error_handler: &'a mut ErrorHandler) -> Self {
        TableBuilder {
            // We add an empty string so when we join the vector with '::' separators, we'll get a leading "::".
            current_scope: vec!["".to_owned()],
            lookup_table: HashMap::new(),
            error_handler,
        }
    }

    /// Computes the fully scoped identifier for the provided element, and stores an entry for it in the lookup table.
    fn add_entry(&mut self, element: &impl NamedSymbol, index: usize, ast: &SliceAst) {
        let scoped_identifier = self.current_scope.join("::") + "::" + element.identifier();

        // Issue an error if the table already contains an entry for this fully scoped identifier.
        if let Some(index) = self.lookup_table.get(&scoped_identifier) {
            let original = ast.resolve_index(*index);

            self.error_handler.report_error((
                "cannot reuse identifier `{}` in this scope",
                element.location(),
            ).into());
            self.error_handler.report_note((
                "",
                original.as_named_symbol().unwrap().location(),
            ).into());
        } else {
            // Otherwise insert the identifier and it's definition's index into the lookup table.
            self.lookup_table.insert(scoped_identifier, index);
        }
    }
}

impl<'a> Visitor for TableBuilder<'a> {
    fn visit_module_start(&mut self, module_def: &Module, index: usize, ast: &SliceAst) {
        self.add_entry(module_def, index, ast);
        self.current_scope.push(module_def.identifier().to_owned());
    }

    fn visit_module_end(&mut self, _: &Module, _: usize, _: &SliceAst) {
        self.current_scope.pop();
    }

    fn visit_struct_start(&mut self, struct_def: &Struct, index: usize, ast: &SliceAst) {
        self.add_entry(struct_def, index, ast);
        self.current_scope.push(struct_def.identifier().to_owned());
    }

    fn visit_struct_end(&mut self, _: &Struct, _: usize, _: &SliceAst) {
        self.current_scope.pop();
    }

    fn visit_interface_start(&mut self, interface_def: &Interface, index: usize, ast: &SliceAst) {
        self.add_entry(interface_def, index, ast);
        self.current_scope.push(interface_def.identifier().to_owned());
    }

    fn visit_interface_end(&mut self, _: &Interface, _: usize, _: &SliceAst) {
        self.current_scope.pop();
    }

    fn visit_data_member(&mut self, data_member: &DataMember, index: usize, ast: &SliceAst) {
        self.add_entry(data_member, index, ast);
    }
}
