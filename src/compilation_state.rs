// Copyright (c) ZeroC, Inc.

use crate::ast::Ast;
use crate::diagnostics::DiagnosticReporter;
use crate::slice_file::SliceFile;
use crate::slice_options::SliceOptions;
use console::Term;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CompilationState {
    pub ast: Ast,
    pub diagnostic_reporter: DiagnosticReporter,
    pub files: HashMap<String, SliceFile>,
}

impl CompilationState {
    pub fn create(options: &SliceOptions) -> Self {
        CompilationState {
            ast: Ast::create(),
            diagnostic_reporter: DiagnosticReporter::new(options),
            files: HashMap::new(),
        }
    }

    /// Calls the provided function on this `CompilationState` if and only if no errors have been emitted so far.
    /// If errors have been reported through this `CompilationState`'s [`DiagnosticReporter`], this is no-op.
    pub fn apply(&mut self, function: fn(&mut Self)) {
        if !self.diagnostic_reporter.has_errors() {
            function(self);
        }
    }

    /// Calls the provided function on this `CompilationState` if and only if no errors have been emitted so far.
    /// If errors have been reported through this `CompilationState`'s [`DiagnosticReporter`], this is no-op.
    ///
    /// # Safety
    ///
    /// The caller of this function must ensure that no (`WeakPtr`s)[crate::utils::ptr_util::WeakPtr] exist that point
    /// to the contents of this `CompilationState`. Even if they're not being actively used, their existence causes UB.
    pub unsafe fn apply_unsafe(&mut self, function: unsafe fn(&mut Self)) {
        if !self.diagnostic_reporter.has_errors() {
            function(self);
        }
    }

    pub fn into_exit_code(self) -> i32 {
        DiagnosticReporter::emit_diagnostics_and_get_exit_code(self, &mut Term::stderr())
    }
}
