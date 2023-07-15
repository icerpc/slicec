// Copyright (c) ZeroC, Inc.

use crate::ast::Ast;
use crate::diagnostic_emitter::DiagnosticEmitter;
use crate::diagnostics::{get_totals, Diagnostic, Diagnostics};
use crate::slice_file::SliceFile;
use crate::slice_options::SliceOptions;
use console::Term;
use std::collections::HashMap;
use std::io::{Result, Write};

#[derive(Debug)]
pub struct CompilationState {
    pub ast: Ast,
    pub diagnostics: Diagnostics,
    pub files: HashMap<String, SliceFile>,
}

impl CompilationState {
    pub fn create() -> Self {
        CompilationState {
            ast: Ast::create(),
            diagnostics: Diagnostics::new(),
            files: HashMap::new(),
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

    pub fn into_exit_code(self, options: &SliceOptions) -> i32 {
        let diagnostics = self.diagnostics.into_updated(&self.ast, &self.files, options);
        let (total_warnings, total_errors) = get_totals(&diagnostics);

        // Print any diagnostics to the console, along with the total number of warnings and errors emitted.
        let mut stderr = Term::stderr();
        let mut emitter = DiagnosticEmitter::new(&mut stderr, options, &self.files);
        DiagnosticEmitter::emit_diagnostics(&mut emitter, diagnostics).expect("failed to emit diagnostics");
        Self::emit_totals(total_warnings, total_errors).expect("failed to emit totals");

        // Return exit code 1 if any errors were emitted, and exit code 0 otherwise.
        i32::from(total_errors != 0)
    }

    /// Consumes this `CompilationState` and returns the diagnostics it contains.
    /// This method exists to simplify the testing of diagnostic emission.
    pub fn into_diagnostics(self, options: &SliceOptions) -> Vec<Diagnostic> {
        self.diagnostics.into_updated(&self.ast, &self.files, options)
    }

    fn emit_totals(total_warnings: usize, total_errors: usize) -> Result<()> {
        let stdout = &mut Term::stdout();

        if total_warnings > 0 {
            let warnings = console::style("Warnings").yellow().bold();
            writeln!(stdout, "{warnings}: Compilation generated {total_warnings} warning(s)")?;
        }
        if total_errors > 0 {
            let failed = console::style("Failed").red().bold();
            writeln!(stdout, "{failed}: Compilation failed with {total_errors} error(s)")?;
        }

        Ok(())
    }
}
