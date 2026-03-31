// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct TypeAlias {
    pub identifier: Identifier,
    pub underlying: TypeRef,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub comment: Option<DocComment>,
    pub span: Span,
}

impl AsTypes for TypeAlias {
    fn concrete_type(&self) -> Types<'_> {
        self.underlying.concrete_type()
    }
}

impl Type for TypeAlias {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }
}

implement_Element_for!(TypeAlias, "type alias");
implement_Attributable_for!(TypeAlias);
implement_Entity_for!(TypeAlias);
implement_Commentable_for!(TypeAlias);
