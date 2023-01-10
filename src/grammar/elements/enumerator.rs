// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct EnumeratorValue {
    pub span: Option<Span>,
    pub value: i128,
}

#[derive(Debug)]
pub struct Enumerator {
    pub identifier: Identifier,
    pub value: EnumeratorValue,
    pub parent: WeakPtr<Enum>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub span: Span,
}

impl Enumerator {
    pub fn value(&self) -> i128 {
        self.value.value
    }
}

implement_Element_for!(EnumeratorValue, "enumerator value");

implement_Element_for!(Enumerator, "enumerator");
implement_Entity_for!(Enumerator);
implement_Contained_for!(Enumerator, Enum);
