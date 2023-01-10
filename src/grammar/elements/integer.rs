// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::slice_file::Span;

#[derive(Debug, PartialEq, Eq)]
pub struct Integer {
    pub value: i128,
    pub span: Span,
}

implement_Element_for!(Integer, "integer");
