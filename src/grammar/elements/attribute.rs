// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Span;

#[derive(Clone, Debug)]
pub struct Attribute {
    pub directive: String,
    pub arguments: Vec<String>,
    pub span: Span,
}

implement_Element_for!(Attribute, "attribute");
implement_Symbol_for!(Attribute);
