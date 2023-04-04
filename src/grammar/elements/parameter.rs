// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct Parameter {
    pub identifier: Identifier,
    pub data_type: TypeRef,
    pub tag: Option<Integer<u32>>,
    pub is_streamed: bool,
    pub is_returned: bool,
    pub parent: WeakPtr<Operation>,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub comment: Option<DocComment>,
    pub span: Span,
}

impl Element for Parameter {
    fn kind(&self) -> &'static str {
        if self.is_returned {
            "return element"
        } else {
            "parameter"
        }
    }
}

implement_Entity_for!(Parameter);
implement_Contained_for!(Parameter, Operation);
implement_Member_for!(Parameter);
