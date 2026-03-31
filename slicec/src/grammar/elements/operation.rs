// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Operation {
    pub identifier: Identifier,
    pub parameters: Vec<WeakPtr<Parameter>>,
    pub return_type: Vec<WeakPtr<Parameter>>,
    pub is_idempotent: bool,
    pub parent: WeakPtr<Interface>,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub comment: Option<DocComment>,
    pub span: Span,
}

impl Operation {
    pub fn parameters(&self) -> Vec<&Parameter> {
        self.parameters.iter().map(WeakPtr::borrow).collect()
    }

    pub fn return_members(&self) -> Vec<&Parameter> {
        self.return_type.iter().map(WeakPtr::borrow).collect()
    }
}

implement_Element_for!(Operation, "operation");
implement_Attributable_for!(@Contained Operation);
implement_Entity_for!(Operation);
implement_Commentable_for!(Operation);
implement_Contained_for!(Operation, Interface);
