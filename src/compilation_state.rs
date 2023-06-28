// Copyright (c) ZeroC, Inc.

use crate::ast::Ast;
use crate::diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticReporter};
use crate::slice_file::{SliceFile, Span};
use crate::slice_options::{DiagnosticFormat, SliceOptions};
use serde::ser::SerializeStruct;
use serde::Serializer;
use std::collections::HashMap;
use std::io::{Result, Write};

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

    pub fn into_exit_code(mut self, output: &mut impl Write) -> i32 {
        // Update the diagnostic levels of all diagnostics based on any attributes or command line options.
        let (total_warnings, total_errors) = self.diagnostic_reporter.update_diagnostics(&self.ast, &self.files);

        // Print the diagnostics to the console, along with the total number of warnings and errors emitted.
        Self::emit_diagnostics(self, output).expect("failed to print diagnostics");
        Self::emit_totals(total_warnings, total_errors).expect("failed to print totals");

        // Return exit code 1 if any errors were reported, and exit code 0 otherwise.
        i32::from(total_errors != 0)
    }

    /// Consumes the diagnostic reporter and writes any non-allowed diagnostics to the provided output.
    fn emit_diagnostics(self, output: &mut impl Write) -> Result<()> {
        // Disable colors if the user requested no colors.
        if self.diagnostic_reporter.disable_color {
            console::set_colors_enabled(false);
            console::set_colors_enabled_stderr(false);
        }

        match self.diagnostic_reporter.diagnostic_format {
            DiagnosticFormat::Human => self.emit_diagnostics_in_human(output),
            DiagnosticFormat::Json => self.emit_diagnostics_in_json(output),
        }
    }

    fn emit_diagnostics_in_json(self, output: &mut impl Write) -> Result<()> {
        // Write each diagnostic as a single line of JSON.
        for diagnostic in self.diagnostic_reporter.diagnostics {
            let severity = match diagnostic.level() {
                DiagnosticLevel::Error => "error",
                DiagnosticLevel::Warning => "warning",
                DiagnosticLevel::Allowed => continue,
            };

            let mut serializer = serde_json::Serializer::new(&mut *output);
            let mut state = serializer.serialize_struct("Diagnostic", 5)?;
            state.serialize_field("message", &diagnostic.message())?;
            state.serialize_field("severity", severity)?;
            state.serialize_field("span", &diagnostic.span())?;
            state.serialize_field("notes", diagnostic.notes())?;
            state.serialize_field("error_code", diagnostic.code())?;
            state.end()?;
            writeln!(output)?; // Separate each diagnostic by a newline character.
        }
        Ok(())
    }

    fn emit_diagnostics_in_human(self, output: &mut impl Write) -> Result<()> {
        fn append_snippet(message: &mut Vec<String>, span: &Span, files: &HashMap<String, SliceFile>) {
            // Display the file name and line row and column where the error began.
            let file_location = format!("{}:{}:{}", &span.file, span.start.row, span.start.col);
            let path = std::path::Path::new(&file_location);
            message.push(format!(" {} {}", console::style("-->").blue().bold(), path.display()));

            // Display the line of code where the error occurred.
            let snippet = files.get(&span.file).unwrap().get_snippet(span.start, span.end);
            message.push(snippet);
        }

        for diagnostic in self.diagnostic_reporter.diagnostics {
            // Style the prefix. Note that for `Notes` we do not insert a newline since they should be "attached"
            // to the previously emitted diagnostic.
            let code = diagnostic.code();
            let prefix = match diagnostic.level() {
                DiagnosticLevel::Error => console::style(format!("error [{code}]")).red().bold(),
                DiagnosticLevel::Warning => console::style(format!("warning [{code}]")).yellow().bold(),
                DiagnosticLevel::Allowed => continue,
            };

            let mut message = vec![];

            // Emit the message with the prefix.
            message.push(format!("{prefix}: {}", console::style(diagnostic.message()).bold()));

            // If the diagnostic contains a span, show a snippet containing the offending code.
            if let Some(span) = diagnostic.span() {
                append_snippet(&mut message, span, &self.files);
            }

            // If the diagnostic contains notes, display them.
            for note in diagnostic.notes() {
                message.push(format!(
                    "    {} {}: {:}",
                    console::style("=").blue().bold(),
                    console::style("note").bold(),
                    console::style(&note.message).bold(),
                ));
                // Only display the snippet if the note has a different span than the diagnostic.
                if note.span.as_ref() != diagnostic.span() {
                    if let Some(span) = &note.span {
                        append_snippet(&mut message, span, &self.files)
                    }
                }
            }
            writeln!(output, "{}", message.join("\n"))?;
        }
        Ok(())
    }

    /// Prints the total number of warnings and errors to stdout.
    /// These messages are conditionally printed; if there were no warnings or errors we don't print them.
    fn emit_totals(total_warnings: usize, total_errors: usize) -> Result<()> {
        let mut stdout = std::io::stdout();

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

    /// Updates and returns all the diagnostics reported during compilation.
    /// This method exists to simplify the testing of diagnostic emission.
    pub fn into_diagnostics(mut self) -> Vec<Diagnostic> {
        // Update and return all the diagnostics.
        self.diagnostic_reporter.update_diagnostics(&self.ast, &self.files);
        self.diagnostic_reporter.diagnostics
    }
}
