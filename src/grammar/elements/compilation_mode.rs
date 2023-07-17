// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;

#[derive(Clone, Debug)]
pub struct FileCompilationMode {
    pub version: CompilationMode,
    pub span: Span,
}

implement_Element_for!(FileCompilationMode, "file compilation mode");
implement_Symbol_for!(FileCompilationMode);
