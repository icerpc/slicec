// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Span;

#[derive(Clone, Debug)]
pub struct Identifier {
    pub value: String,
    pub span: Span,
}

implement_Element_for!(Identifier, "identifier");
implement_Symbol_for!(Identifier);
