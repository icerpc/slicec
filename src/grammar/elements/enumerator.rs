// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::ptr_util::WeakPtr;
use crate::slice_file::Span;

#[derive(Debug)]
pub struct Enumerator {
    pub identifier: Identifier,
    pub value: i64,
    pub parent: WeakPtr<Enum>,
    pub scope: Scope,
    pub attributes: Vec<Attribute>,
    pub comment: Option<DocComment>,
    pub span: Span,
}

impl Enumerator {
    pub(crate) fn new(
        identifier: Identifier,
        value: i64,
        scope: Scope,
        attributes: Vec<Attribute>,
        comment: Option<DocComment>,
        span: Span,
    ) -> Self {
        let parent = WeakPtr::create_uninitialized();
        Enumerator {
            identifier,
            value,
            parent,
            scope,
            attributes,
            comment,
            span,
        }
    }
}

implement_Element_for!(Enumerator, "enumerator");
implement_Entity_for!(Enumerator);
implement_Contained_for!(Enumerator, Enum);
