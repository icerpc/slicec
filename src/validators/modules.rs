// Copyright (c) ZeroC, Inc.

use std::collections::HashMap;

use crate::ast::node::Node;
use crate::compilation_state::CompilationState;
use crate::diagnostics::{Diagnostic, Error};
use crate::grammar::*;

/// Since modules can be re-opened, but each module is a distinct element in the AST, our normal redefinition check
/// is inadequate. If 2 modules have the same name we have to check for redefinitions across both modules.
///
/// So we compute a map of all the contents in modules with the same name (fully scoped), then check that.
// TODO fix this function! modules can't contain other modules now!
pub fn validate_module_contents(compilation_state: &mut CompilationState) {
    let mut merged_module_contents: HashMap<String, Vec<&Definition>> = HashMap::new();
    for node in compilation_state.ast.as_slice() {
        if let Node::Module(module_ptr) = node {
            // Borrow the module's pointer and store its fully scoped identifier.
            let module = module_ptr.borrow();
            let scoped_module_identifier = module.parser_scoped_identifier();

            // Add the contents to the map, with the module's scoped identifier as the key.
            merged_module_contents
                 .entry(scoped_module_identifier)
                 .or_default() // If an entry doesn't exist for the key, create one now.
                 .extend(module.contents()); // Add this module's contents to the existing vector.
        }
    }

    for mut module_contents in merged_module_contents.into_values() {
        // Sort the contents by identifier first so that we can use windows to search for duplicates.
        module_contents.sort_by_key(|def| def.borrow().raw_identifier().value.to_owned());
        module_contents.windows(2).for_each(|window| {
            let identifier_0 = window[0].borrow().raw_identifier();
            let identifier_1 = window[1].borrow().raw_identifier();

            // We don't want to report a redefinition error if both definitions are modules, since
            // that's allowed. If both identifiers are the same and either definition is not a module, then we have a
            // redefinition error.
            if identifier_0.value == identifier_1.value
            // TODO: && !(matches!(window[0], Definition::Module(_)) && matches!(window[1], Definition::Module(_)))
            {
                Diagnostic::new(Error::Redefinition {
                    identifier: identifier_1.value.clone(),
                })
                .set_span(identifier_1.span())
                .add_note(
                    format!("'{}' was previously defined here", identifier_0.value),
                    Some(identifier_0.span()),
                )
                .report(&mut compilation_state.diagnostic_reporter);
            }
        });
    }
}
