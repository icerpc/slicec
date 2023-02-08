// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;

#[derive(Debug)]
pub struct Integer {
    pub value: i128,
    pub span: Span,
}

implement_Element_for!(Integer, "integer");
implement_Symbol_for!(Integer);
