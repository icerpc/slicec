// Copyright (c) ZeroC, Inc.

use crate::ast::Ast;
use crate::diagnostics::{AnnotatedDiagnostic, Diagnostics};
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
    /// The caller of this function must ensure that no [`WeakPtr`](crate::utils::ptr_util::WeakPtr)s exist that point
    /// to the contents of this `CompilationState`. Even if they're not being actively used, their existence causes UB.
    pub unsafe fn apply_unsafe(&mut self, function: unsafe fn(&mut Self)) {
        if !self.diagnostics.has_errors() {
            function(self);
        }
    }

    /// Creates an [`AnnotatedDiagnostic`] for each [`Diagnostic`](crate::diagnostics::Diagnostic) stored in this
    /// `CompilationState`. This does not "use up" the diagnostics. Calling this function multiple times will yield the
    /// same output.
    ///
    /// The returned diagnostics are also de-duplicated. Duplicates are expected when slicec calls multiple plugins
    /// which may run identical validation. If multiple diagnostics are _exact_ duplicates, only the first will be
    /// present in the returned [`Vec`].
    pub fn get_annotated_diagnostics(&self, options: &SliceOptions) -> Vec<AnnotatedDiagnostic> {
        let mut annotated_diagnostics = Vec::<AnnotatedDiagnostic>::new();
        for diagnostic in &*self.diagnostics {
            let converted = crate::diagnostics::convert_diagnostic(diagnostic, options, &self.ast, &self.files);
            if let Some(original) = annotated_diagnostics.iter_mut().find(|d| **d == converted) {
                original.reported_by.extend(converted.reported_by);
            } else {
                annotated_diagnostics.push(converted);
            }
        }
        annotated_diagnostics
    }
}
