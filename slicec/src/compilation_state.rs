// Copyright (c) ZeroC, Inc.

use crate::ast::Ast;
use crate::diagnostics::{Diagnostics, AnnotatedDiagnostic};
use crate::slice_file::SliceFile;
use crate::slice_options::SliceOptions;

#[derive(Debug, Default)]
pub struct CompilationState {
    pub ast: Ast,
    pub diagnostics: Diagnostics,
    pub files: Vec<SliceFile>,
}

impl CompilationState {
    pub fn create() -> Self {
        CompilationState {
            ast: Ast::create(),
            diagnostics: Diagnostics::new(),
            files: Vec::new(),
        }
    }

    /// Calls the provided function on this `CompilationState` if and only if no errors have been reported so far.
    /// If any errors are present in this `CompilationState`'s [Diagnostics] container, this is no-op.
    pub fn apply(&mut self, function: fn(&mut Self)) {
        if !self.diagnostics.has_errors() {
            function(self);
        }
    }

    /// Calls the provided function on this `CompilationState` if and only if no errors have been reported so far.
    /// If any errors are present in this `CompilationState`'s [Diagnostics] container, this is no-op.
    ///
    /// # Safety
    ///
    /// The caller of this function must ensure that no (`WeakPtr`s)[crate::utils::ptr_util::WeakPtr] exist that point
    /// to the contents of this `CompilationState`. Even if they're not being actively used, their existence causes UB.
    pub unsafe fn apply_unsafe(&mut self, function: unsafe fn(&mut Self)) {
        if !self.diagnostics.has_errors() {
            function(self);
        }
    }

    pub fn get_annotated_diagnostics(&self, options: &SliceOptions) -> Vec<AnnotatedDiagnostic> {
        self.diagnostics
            .iter()
            .map(|diagnostic| crate::diagnostics::convert_diagnostic(diagnostic, options, self))
            .collect()
    }
}
