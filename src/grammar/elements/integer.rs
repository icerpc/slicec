// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Integer<T: Debug> {
    pub value: T,
    pub span: Span,
}

implement_Element_for!(Integer<T>, "integer", Debug);
implement_Symbol_for!(Integer<T>, Debug);
