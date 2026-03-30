// Copyright (c) ZeroC, Inc.

use super::Module;
use crate::utils::ptr_util::WeakPtr;

#[derive(Clone, Debug, Default)]
pub struct Scope {
    pub parser_scope: String,
    pub module: Option<WeakPtr<Module>>,
}

impl Scope {
    pub fn push_scope(&mut self, scope: &str) {
        if !self.parser_scope.is_empty() {
            self.parser_scope.push_str("::");
        }
        self.parser_scope.push_str(scope);
    }

    pub fn pop_scope(&mut self) {
        if let Some(last_scope_index) = self.parser_scope.rfind("::") {
            // Remove any characters after the last '::' in the string.
            // We ensure that we're only removing additional parser scopes, and not any scopes that came from a module.
            #[cfg(debug_assertions)]
            {
                let module_scope = self.module.as_ref().map(|m| m.borrow().nested_module_identifier());
                debug_assert!(self.parser_scope.len() > module_scope.map_or(0, str::len))
            }
            self.parser_scope.truncate(last_scope_index);
        } else {
            // If the string doesn't contain '::', there's only a single scope. We pop it off by clearing the string.
            // This is only possible if we're not in a module, otherwise we'd always have at least 1 module scope.
            debug_assert!(self.module.is_none());
            self.parser_scope.clear();
        }
    }
}

/// Returns the scoped version of the provided identifier.
pub fn get_scoped_identifier(identifier: &str, scope: &str) -> String {
    if scope.is_empty() {
        identifier.to_owned()
    } else {
        scope.to_owned() + "::" + identifier
    }
}
