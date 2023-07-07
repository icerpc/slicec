// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;

#[derive(Clone, Debug)]
pub struct FileMode {
    pub mode: Mode,
    pub span: Span,
}

implement_Element_for!(FileMode, "file mode");
implement_Symbol_for!(FileMode);
