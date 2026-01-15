// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Module {
    pub identifier: Identifier,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub span: Span,
}

impl Module {
    /// If this module was declared using nested module syntax, this returns the entire nested identifier.
    /// Otherwise this just returns the module's identifier.
    pub fn nested_module_identifier(&self) -> &str {
        &self.identifier.value
    }
}

impl NamedSymbol for Module {
    fn identifier(&self) -> &str {
        // If this module uses nested module syntax, only return the last segment (corresponds to the innermost module).
        if let Some(last_colon_index) = self.identifier.value.rfind(':') {
            &self.identifier.value[last_colon_index + 1..]
        } else {
            &self.identifier.value
        }
    }

    fn raw_identifier(&self) -> &Identifier {
        &self.identifier
    }

    fn module_scoped_identifier(&self) -> String {
        self.nested_module_identifier().to_owned()
    }

    fn parser_scoped_identifier(&self) -> String {
        self.nested_module_identifier().to_owned()
    }
}

implement_Element_for!(Module, "module");
implement_Symbol_for!(Module);
implement_Attributable_for!(Module);
