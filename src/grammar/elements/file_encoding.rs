// Copyright (c) ZeroC, Inc. All rights reserved.

use super::super::*;
use crate::utils::slice_file::Span;

#[derive(Clone, Debug)]
pub struct FileEncoding {
    pub version: Encoding,
    pub span: Span,
}

implement_Element_for!(FileEncoding, "file encoding");
implement_Symbol_for!(FileEncoding);
