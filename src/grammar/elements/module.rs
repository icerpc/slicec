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

// TODO do we return the full identifier or the last identifier here?
impl NamedSymbol for Module {
    fn identifier(&self) -> &str {
        &self.identifier.value
    }

    fn raw_identifier(&self) -> &Identifier {
        &self.identifier
    }

    fn module_scoped_identifier(&self) -> String {
        self.identifier().to_owned()
    }

    fn parser_scoped_identifier(&self) -> String {
        self.identifier().to_owned()
    }
}

implement_Element_for!(Module, "module");
implement_Symbol_for!(Module);
implement_Attributable_for!(Module);
