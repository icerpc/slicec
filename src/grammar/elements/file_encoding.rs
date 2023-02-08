// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;

#[derive(Clone, Debug)]
pub struct FileEncoding {
    pub version: Encoding,
    pub span: Span,
}

implement_Element_for!(FileEncoding, "file encoding");
implement_Symbol_for!(FileEncoding);
